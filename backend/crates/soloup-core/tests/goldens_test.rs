//! §10.1 数值正确性 —— 黄金表快照（C/V 3 位、E 1 位、等级 2 位）+ 断更 + 闭合等价。
//! 对应 TS `@soloup/core/settle.test.ts` 的黄金表与断更用例。

use soloup_core::curve::{level_from_category, level_of, resolve_curve};
use soloup_core::goldens::{golden_continuous, GOLDEN_COGNITIVE_GAP};
use soloup_core::params::category_account_params;
use soloup_core::settle::{apply_gap_closed_form, replay_days, settle_day, Accounts, SettleParams, ZERO_ACCOUNTS};
use soloup_core::{Difficulty, SkillCategory};

fn round(x: f64, dp: i32) -> f64 {
    let f = 10f64.powi(dp);
    (x * f).round() / f
}

fn params_of(cat: SkillCategory) -> SettleParams {
    let (c, f) = category_account_params(cat);
    SettleParams { c, f }
}

fn cat_name(cat: SkillCategory) -> &'static str {
    match cat {
        SkillCategory::Physical => "physical",
        SkillCategory::Cognitive => "cognitive",
        SkillCategory::Knowledge => "knowledge",
    }
}

fn assert_eq_round(actual: f64, expected: f64, dp: i32, what: &str, cat: &str, day: i32) {
    let r = round(actual, dp);
    assert!(
        (r - expected).abs() < 1e-9,
        "{}[{}] day {}: round(actual, {})={} != expected={}",
        cat,
        what,
        day,
        dp,
        r,
        expected
    );
}

#[test]
fn golden_continuous_matches() {
    for cat in SkillCategory::ALL {
        let name = cat_name(cat);
        let p = params_of(cat);
        let checkins = vec![true; 1825];
        let (_, per_day) = replay_days(ZERO_ACCOUNTS, p, &checkins);

        for row in golden_continuous(cat) {
            let acc: Accounts = if row.day == 0 {
                ZERO_ACCOUNTS
            } else {
                per_day[(row.day - 1) as usize]
            };
            let e = acc.c + acc.v;
            let level = level_from_category(e, cat, Difficulty::Normal, None);
            assert_eq_round(acc.c, row.c, 3, "c", name, row.day);
            assert_eq_round(acc.v, row.v, 3, "v", name, row.day);
            assert_eq_round(e, row.e, 1, "e", name, row.day);
            assert_eq_round(level, row.level, 2, "level", name, row.day);
        }
    }
}

#[test]
fn golden_cognitive_gap_matches() {
    let cat = SkillCategory::Cognitive;
    let p = params_of(cat);
    let curve = resolve_curve(cat, None, Difficulty::Normal);

    // 打卡 100 天
    let checkins = vec![true; 100];
    let (day100, _) = replay_days(ZERO_ACCOUNTS, p, &checkins);
    let e100 = day100.c + day100.v;
    assert_eq_round(day100.c, GOLDEN_COGNITIVE_GAP.day100_c, 3, "c", "cognitive", 100);
    assert_eq_round(day100.v, GOLDEN_COGNITIVE_GAP.day100_v, 3, "v", "cognitive", 100);
    assert_eq_round(e100, GOLDEN_COGNITIVE_GAP.day100_e, 1, "e", "cognitive", 100);
    assert_eq_round(
        level_of(e100, &curve),
        GOLDEN_COGNITIVE_GAP.day100_level,
        2,
        "level",
        "cognitive",
        100,
    );

    // 断更 60 天（闭合公式）
    let day160 = apply_gap_closed_form(day100, p, 60.0);
    let e160 = day160.c + day160.v;
    assert_eq_round(day160.c, GOLDEN_COGNITIVE_GAP.day160_c, 3, "c", "cognitive", 160);
    assert_eq_round(day160.v, GOLDEN_COGNITIVE_GAP.day160_v, 3, "v", "cognitive", 160);
    assert_eq_round(e160, GOLDEN_COGNITIVE_GAP.day160_e, 1, "e", "cognitive", 160);
    assert_eq_round(
        level_of(e160, &curve),
        GOLDEN_COGNITIVE_GAP.day160_level,
        2,
        "level",
        "cognitive",
        160,
    );

    // 语义检查：结晶仍增、活性回落
    assert!(day160.c > day100.c);
    assert!(day160.v < day100.v);
}

#[test]
fn gap_closed_form_equals_daily_iteration() {
    let cat = SkillCategory::Knowledge;
    let p = params_of(cat);
    let checkins = vec![true; 100];
    let (start, _) = replay_days(ZERO_ACCOUNTS, p, &checkins);

    let closed = apply_gap_closed_form(start, p, 60.0);

    let mut daily = start;
    for _ in 0..60 {
        daily = settle_day(daily, p, false);
    }

    let rel_c = (closed.c - daily.c).abs() / daily.c;
    let rel_v = (closed.v - daily.v).abs() / daily.v;
    assert!(rel_c <= 1e-9, "relC={rel_c}");
    assert!(rel_v <= 1e-9, "relV={rel_v}");
}
