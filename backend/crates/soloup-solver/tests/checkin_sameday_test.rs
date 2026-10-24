//! 回归：打卡当日必须立刻计入账户。
//!
//! 曾经的缺陷：`checkin` 在事务**之前**取打卡快照，事务内先写当日记录再拿旧快照重放，
//! 于是当天这一次永远不可见；而 `last_settled_date` 已推进到当日，增量推进不再回头，
//! 该日永久漏算（唯一一次打卡的技能直接 c=v=0）。三条用例分别盯：
//! 1) 当日入账；2) 同日二次编辑不叠加；3) 次日打卡不吞前一日。

use chrono::NaiveDate;

use soloup_core::schema::{AuditActor, Difficulty, Skill, SkillCategory};
use soloup_solver::service::{CheckinInput, Solver};
use soloup_store::OpenStoreOptions;

fn mem_store() -> soloup_store::Store {
    soloup_store::open_store(OpenStoreOptions {
        path: Some(":memory:".to_string()),
    })
    .expect("open")
}

fn leaf(id: &str, name: &str, created: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        parent_id: None,
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
        is_branch: false,
    }
}

fn checkin(solver: &mut Solver, date: &str, id: &str) -> Vec<soloup_solver::service::SkillEffect> {
    solver
        .checkin(
            &CheckinInput {
                date: date.to_string(),
                skill_ids: vec![id.to_string()],
                project_id: None,
                note: None,
            },
            AuditActor::User,
        )
        .expect("checkin")
}

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn same_day_checkin_is_accounted_immediately() {
    let store = mem_store();
    store
        .skills()
        .create(&leaf("obs", "观察与记录", "2026-01-01"))
        .unwrap();
    let mut solver = Solver::with_now(store, day(2026, 1, 1));

    let effects = checkin(&mut solver, "2026-01-01", "obs");
    assert_eq!(effects.len(), 1);
    let e = &effects[0];
    // cognitive：c=0.01 / f=0.008。单日打卡 c = 0.01×1，v = 1×(1−0.018)。
    assert!(
        (e.after.c - 0.01).abs() < 1e-9,
        "当日打卡 c 应为 0.01，实际 {}",
        e.after.c
    );
    assert!(
        (e.after.v - 0.982).abs() < 1e-9,
        "当日打卡 v 应为 0.982，实际 {}",
        e.after.v
    );
}

#[test]
fn same_day_second_checkin_does_not_stack() {
    let store = mem_store();
    store
        .skills()
        .create(&leaf("obs", "观察与记录", "2026-01-01"))
        .unwrap();
    let mut solver = Solver::with_now(store, day(2026, 1, 1));

    checkin(&mut solver, "2026-01-01", "obs");
    let effects = checkin(&mut solver, "2026-01-01", "obs");
    let e = &effects[0];
    assert!(
        (e.after.c - 0.01).abs() < 1e-9,
        "同日重复提交应幂等，实际 c={}",
        e.after.c
    );
    assert!((e.after.v - 0.982).abs() < 1e-9);
}

#[test]
fn next_day_checkin_keeps_previous_day() {
    let store = mem_store();
    store
        .skills()
        .create(&leaf("obs", "观察与记录", "2026-01-01"))
        .unwrap();
    let mut solver = Solver::with_now(store, day(2026, 1, 1));

    checkin(&mut solver, "2026-01-01", "obs");
    solver.set_now(day(2026, 1, 2));
    let effects = checkin(&mut solver, "2026-01-02", "obs");

    // 连续两天：c = 0.01×1 + 0.01×1.982 = 0.02982；v = 1.982×0.982 = 1.946324
    let e = &effects[0];
    assert!(
        (e.after.c - 0.02982).abs() < 1e-9,
        "两天连续打卡 c 应为 0.02982，实际 {}",
        e.after.c
    );
    assert!(
        (e.after.v - 1.946324).abs() < 1e-9,
        "两天连续打卡 v 应为 1.946324，实际 {}",
        e.after.v
    );
}
