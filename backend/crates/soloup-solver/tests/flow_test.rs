//! solver 端到端冒烟：建树 → 100 天打卡 → settle_all 结算 == 黄金表 → snapshot 派生。
//! 对应用例：cognitive 叶子技能连续 100 天打卡（§10.1 黄金表 day100）。

use chrono::NaiveDate;

use soloup_core::dates::to_iso_date;
use soloup_core::schema::{Attribute, AttributeCategory, Difficulty, Skill, SkillCategory};
use soloup_solver::service::Solver;
use soloup_store::daily::DailySaveInput;
use soloup_store::OpenStoreOptions;

fn mem_store() -> soloup_store::Store {
    soloup_store::open_store(OpenStoreOptions {
        path: Some(":memory:".to_string()),
    })
    .expect("open")
}

const D0: &str = "2026-01-01";

fn skill(id: &str, name: &str, parent: Option<&str>, created: &str) -> Skill {
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
        created_at: created.to_string(),
        archived_at: None,
        color: None,
        icon: None,
        sort: 0,
    }
}

#[test]
fn continuous_100_days_matches_golden() {
    let mut store = mem_store();
    let repo = store.attributes();
    let vit = repo
        .create(&Attribute {
            id: String::new(),
            name: "智力".to_string(),
            description: None,
            base_value: 0.0,
            max_value: 100.0,
            alpha: 1.2,
            w0: 400.0,
            category: Some(AttributeCategory::Mental),
            color: None,
            icon: None,
            sort: 0,
        })
        .unwrap();

    store.skills().create(&skill("tech", "技术", None, D0)).unwrap();
    store.skills().create(&skill("fe", "前端", Some("tech"), D0)).unwrap();

    // fe ↔ 智力 关联
    store.tx(|c| {
        soloup_store::links::LinkRepo { db: c }.set_for_skill(
            "fe",
            &[soloup_store::links::SkillLinkInput {
                attribute_id: vit.id.clone(),
                weight: 1.0,
            }],
        )
    })
    .unwrap();

    // 100 天每天打卡 fe（2026-01-01 .. 2026-04-10）
    let mut solver = Solver::with_now(store, NaiveDate::from_ymd_opt(2026, 4, 10).unwrap());
    let mut d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    for _ in 0..100 {
        let iso = to_iso_date(d);
        solver
            .store
            .tx(|c| {
                soloup_store::daily::DailyRepo { db: c }.save(
                    &iso,
                    &DailySaveInput {
                        project_id: None,
                        note: None,
                        skill_ids: vec!["fe".to_string()],
                    },
                )
            })
            .unwrap();
        d += chrono::Duration::days(1);
    }

    // 结算（当前"今天" = 4/10，第 100 天）
    let effects = solver.settle_all().unwrap();
    assert_eq!(effects.len(), 1);
    let effect = &effects[0];

    // 黄金表 cognitive day100：c=30.175, v=45.684（round3）
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    assert_eq!(r3(effect.after.c), 30.175);
    assert_eq!(r3(effect.after.v), 45.684);

    // snapshot：fe 叶子等级 round2 == 10.84；智力属性有派生值
    let snap = solver.snapshot(None).unwrap();
    let fe = snap.leaves.iter().find(|l| l.skill.id == "fe").expect("fe leaf");
    let r2 = (fe.level * 100.0).round() / 100.0;
    assert_eq!(r2, 10.84);

    let int = snap.attributes.iter().find(|a| a.attribute.id == vit.id).unwrap();
    assert!(int.value > 0.0);
    assert_eq!(int.entries.len(), 1);
}
