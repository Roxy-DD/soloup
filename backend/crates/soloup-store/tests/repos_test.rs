//! Skills 树 / Daily 打卡 / Links 关联 的综合仓储测试。

use soloup_core::schema::{Attribute, AttributeCategory, Difficulty, Skill, SkillCategory};
use soloup_store::daily::DailySaveInput;
use soloup_store::errors::StoreErrorCode;
use soloup_store::links::SkillLinkInput;
use soloup_store::{open_store, OpenStoreOptions};

fn mem_store() -> soloup_store::Store {
    open_store(OpenStoreOptions {
        path: Some(":memory:".to_string()),
    })
    .expect("open")
}

fn attr(name: &str) -> Attribute {
    Attribute {
        id: String::new(),
        name: name.to_string(),
        description: None,
        base_value: 0.0,
        max_value: 100.0,
        alpha: 1.2,
        w0: 400.0,
        category: Some(AttributeCategory::Physical),
        color: None,
        icon: None,
        sort: 0,
    }
}

fn skill(id: &str, name: &str, parent: Option<&str>) -> Skill {
    Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        parent_id: parent.map(|p| p.to_string()),
        category: SkillCategory::Cognitive,
        difficulty: Difficulty::Normal,
        curve_type: None,
        c: 0.0,
        v: 0.0,
        last_settled_date: None,
        created_at: "2026-01-01".to_string(),
        archived_at: None,
        color: None,
        icon: None,
        sort: 0,
    }
}

#[test]
fn skill_tree_create_child_and_cycle_detection() {
    let s = mem_store();
    let skills = s.skills();
    skills.create(&skill("art", "艺术", None)).unwrap();
    skills.create(&skill("paint", "绘画", Some("art"))).unwrap();
    skills.create(&skill("sketch", "素描", Some("paint"))).unwrap();

    // 子技能 → 孙
    assert_eq!(skills.children_of("art").unwrap().len(), 1);
    assert_eq!(skills.subtree_ids("art").unwrap().len(), 3);

    // 环：把 "paint" 移动成自己后代 "sketch" 的父 → 拒绝
    let mut moved = skills.get("paint").unwrap().unwrap();
    moved.parent_id = Some("sketch".to_string());
    let err = skills.update(&moved).unwrap_err();
    assert_eq!(err.code, StoreErrorCode::Cycle);

    // 把自己设为父级 → 拒绝
    let mut selfmove = skills.get("art").unwrap().unwrap();
    selfmove.parent_id = Some("art".to_string());
    assert_eq!(skills.update(&selfmove).unwrap_err().code, StoreErrorCode::Cycle);

    // 移到不存在的父 → NotFound
    let mut bad = skills.get("art").unwrap().unwrap();
    bad.parent_id = Some("nope".to_string());
    assert_eq!(skills.update(&bad).unwrap_err().code, StoreErrorCode::NotFound);

    // 物理删除有子技能的父 → HasChildren
    assert_eq!(
        skills.delete("art").unwrap_err().code,
        StoreErrorCode::HasChildren
    );

    // 祖先链（根在前）
    assert_eq!(skills.ancestors_of("sketch").unwrap(), vec!["art", "paint"]);
}

#[test]
fn daily_save_clear_and_links() {
    let mut s = mem_store();
    let a = s.attributes().create(&attr("体质")).unwrap();

    // 建树：叶子 "run"（根 art → body → run）
    s.skills().create(&skill("art", "艺术", None)).unwrap();
    s.skills().create(&skill("body", "身体", Some("art"))).unwrap();
    s.skills().create(&skill("run", "跑步", Some("body"))).unwrap();

    // 属性关联只能挂叶子（body 非叶子 → Constraint）
    let tx = s.conn.transaction().unwrap();
    let links = soloup_store::links::LinkRepo { db: &tx };
    let e = links
        .set_for_skill(
            "body",
            &[SkillLinkInput {
                attribute_id: a.id.clone(),
                weight: 0.5,
            }],
        )
        .unwrap_err();
    assert_eq!(e.code, StoreErrorCode::Constraint);
    tx.commit().unwrap();

    // 叶子关联 OK（在事务内）
    s.tx(|c| {
        soloup_store::links::LinkRepo { db: c }.set_for_skill(
            "run",
            &[SkillLinkInput {
                attribute_id: a.id.clone(),
                weight: 0.7,
            }],
        )
    })
    .unwrap();
    assert_eq!(s.links().list_for_skill("run").unwrap().len(), 1);

    // 打卡 + 同日覆盖
    s.tx(|c| {
        soloup_store::daily::DailyRepo { db: c }.save(
            "2026-01-01",
            &DailySaveInput {
                project_id: None,
                note: Some("第一次".to_string()),
                skill_ids: vec!["run".to_string(), "run".to_string(), "missing".to_string()],
            },
        )
    })
    .unwrap_err(); // 含不存在的技能 → 整体报错（事务内无部分写入）

    s.tx(|c| {
        soloup_store::daily::DailyRepo { db: c }.save(
            "2026-01-01",
            &DailySaveInput {
                project_id: None,
                note: None,
                skill_ids: vec!["run".to_string()],
            },
        )
    })
    .unwrap();
    assert_eq!(s.daily().skill_ids_on("2026-01-01").unwrap(), vec!["run"]);

    // 有打卡历史的叶子物理删除 → HasHistory
    assert_eq!(s.skills().delete("run").unwrap_err().code, StoreErrorCode::HasHistory);

    // 归档后 list 只含未归档（art + body = 2）
    s.skills().archive("run", None).unwrap();
    assert_eq!(s.skills().get("run").unwrap().unwrap().archived_at.is_some(), true);
    assert_eq!(s.skills().list().unwrap().len(), 2);

    // 清空打卡后再删除
    s.daily().clear("2026-01-01").unwrap();
    s.skills().restore("run").unwrap();
    s.skills().delete("run").unwrap();
    assert!(s.skills().get("run").unwrap().is_none());
}

#[test]
fn reset_wipe_keeps_selected_settings() {
    let s = mem_store();
    s.settings().set("keep-me", "1").unwrap();
    s.settings().set("drop-me", "2").unwrap();
    s.reset().wipe(&["keep-me"]).unwrap();
    assert_eq!(s.settings().get("keep-me").unwrap(), Some("1".to_string()));
    assert_eq!(s.settings().get("drop-me").unwrap(), None);
}
