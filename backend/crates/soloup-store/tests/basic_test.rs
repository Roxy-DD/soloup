//! store 层冒烟：内存库打开 + 迁移 + Attribute/Settings/Audit CRUD。

use soloup_core::schema::Attribute;
use soloup_store::open_store;
use soloup_store::OpenStoreOptions;
use serde_json::json;

fn mem_store() -> soloup_store::Store {
    open_store(OpenStoreOptions {
        path: Some(":memory:".to_string()),
    })
    .expect("open :memory:")
}

fn sample_attr() -> Attribute {
    Attribute {
        id: String::new(),
        name: "体质".to_string(),
        description: Some("身体的力量".to_string()),
        base_value: 0.0,
        max_value: 100.0,
        alpha: 1.2,
        w0: 400.0,
        category: Some(soloup_core::schema::AttributeCategory::Physical),
        color: Some("#C6352B".to_string()),
        icon: None,
        sort: 0,
    }
}

#[test]
fn migrate_sets_schema_version() {
    let s = mem_store();
    let v: i64 = s
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, soloup_store::latest_schema_version());
}

#[test]
fn attribute_crud_roundtrip() {
    let s = mem_store();
    let repo = s.attributes();
    let created = repo.create(&sample_attr()).unwrap();
    assert!(!created.id.is_empty());
    let id = created.id.clone();

    let got = repo.get(&id).unwrap().expect("should exist");
    assert_eq!(got.name, "体质");
    assert_eq!(got.category, Some(soloup_core::schema::AttributeCategory::Physical));

    let mut updated = got.clone();
    updated.name = "力量".to_string();
    repo.update(&updated).unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().name, "力量");

    assert_eq!(repo.list().unwrap().len(), 1);

    // 删除前有技能关联会被拒 —— 这里无关联，删除成功
    repo.delete(&id).unwrap();
    assert!(repo.get(&id).unwrap().is_none());
}

#[test]
fn settings_json_roundtrip() {
    let s = mem_store();
    let repo = s.settings();
    repo.set_json("param_overrides", &json!({"attr_defaults": {"alpha": 1.3}}))
        .unwrap();
    let v: serde_json::Value = repo.get_json("param_overrides").unwrap().unwrap();
    assert_eq!(v["attr_defaults"]["alpha"], 1.3);
    assert!(repo.get_json::<serde_json::Value>("missing").unwrap().is_none());
}

#[test]
fn audit_append_and_list() {
    let s = mem_store();
    let repo = s.audit();
    repo.append(
        soloup_core::schema::AuditActor::Ai,
        Some("soloup_attribute_create"),
        Some(&json!({"name": "体质"})),
    )
    .unwrap();
    repo.append(soloup_core::schema::AuditActor::User, None, None).unwrap();

    let all = repo.list(&Default::default()).unwrap();
    assert_eq!(all.len(), 2);
    // 倒序：最新在前 → all[0] 是第二次 append（User/None）
    assert_eq!(all[0].actor, soloup_core::schema::AuditActor::User);
    assert_eq!(all[1].tool.as_deref(), Some("soloup_attribute_create"));
    assert_eq!(all[1].params_json.as_ref().unwrap()["name"], "体质");
}
