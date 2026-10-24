//! V7 迁移回归测试：专治「V6 空跑」。
//!
//! 真实库里 `user_version` 已经到 6，但 `dawn` / `grand` / `tenk` 三条成就仍是旧文案 —— 原因是
//! V6 的 `WHERE` 拿**序列化后的 JSON 字符串**做等值比较，而库里键顺序是 `operator/stat/value`，
//! V6 写的是 `stat/operator/value`，字符串不等 → 一条都没匹配上，版本号却照样前进了。
//!
//! 这里把那种库原样造出来（跑到 V6、写入键顺序不同的旧数据、`user_version=6`），
//! 再用 `open_store()` 打开，断言：
//! ① 三条成就的名称 / 描述 / 条件都被改成 seed 的目标态；
//! ② 稀有度归一到分值分档（「完稿 150」由传说降为史诗）；
//! ③ 已经正确的库不会被二次改动（幂等）。

use std::path::{Path, PathBuf};

use soloup_core::schema::DbEnum;
use soloup_store::migrations;
use soloup_store::{open_store, OpenStoreOptions};

fn temp_db(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("soloup-v7-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("soloup.db")
}

/// 造一个「已跑到 V6、但 V6 空跑」的库：旧文案 + 旧条件 + 乱序稀有度。
///
/// 注意 `V4` 会先插入这 9 条并**按 `stat` 在前的键顺序**写条件，于是全新库上 V6 能匹配上；
/// 而真实库里的条件是应用层经 `serde_json::Value` 序列化的**字母序**（`operator` 在前），
/// V6 的字符串等值比较就匹配不上。所以这里必须把条件回写成字母序，才是真库的样子。
fn make_v6_skipped_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).expect("open raw db");
    for m in migrations::MIGRATIONS.iter().filter(|m| m.version <= 6) {
        conn.execute_batch(m.sql)
            .unwrap_or_else(|e| panic!("apply v{} failed: {e}", m.version));
    }
    conn.execute_batch(
        r#"
UPDATE achievements SET description='任一技能达到 4 级',
  condition_json='{"operator":">=","stat":"maxSkillLv","value":4}',
  rarity='rare', points=40, type='skill' WHERE id='dawn';
UPDATE achievements SET name='宗师之路', description='任一技能达到 20 级',
  condition_json='{"operator":">=","stat":"maxSkillLv","value":20}',
  rarity='legendary', points=200, type='skill' WHERE id='grand';
UPDATE achievements SET name='万时之功', description='某个技能累计使用 10,000 次',
  condition_json='{"operator":">=","stat":"maxSkillCheckins","value":10000}',
  rarity='legendary', points=300, type='skill' WHERE id='tenk';
UPDATE achievements SET name='完稿', description='完成第一个项目',
  condition_json='{"operator":">=","stat":"projectsCompleted","value":1}',
  rarity='legendary', points=150, type='project' WHERE id='done1';
UPDATE achievements SET name='初次觉醒', description='完成第一次每日记录',
  condition_json='{"operator":">=","stat":"totalDays","value":1}',
  rarity='common', points=10, type='milestone' WHERE id='first';
"#,
    )
    .expect("rewrite achievements into legacy shape");
    conn.pragma_update(None, "user_version", 6)
        .expect("set user_version to 6");
}

fn open_at(path: &Path) -> soloup_store::Store {
    open_store(OpenStoreOptions {
        path: Some(path.to_string_lossy().to_string()),
    })
    .expect("open store")
}

fn row(store: &soloup_store::Store, id: &str) -> soloup_core::schema::Achievement {
    store
        .achievements()
        .list()
        .expect("list achievements")
        .into_iter()
        .find(|a| a.id == id)
        .unwrap_or_else(|| panic!("achievement {id} missing"))
}

#[test]
fn legacy_conditions_are_rewritten_to_observable_milestones() {
    let db = temp_db("rewrite");
    make_v6_skipped_db(&db);

    let store = open_at(&db);

    // ① 等级条件 → 打卡天数条件，名称与描述同步（原 V6 的意图）
    let dawn = row(&store, "dawn");
    assert_eq!(dawn.description.as_deref(), Some("任一技能累计记录 7 天"));
    let dawn_cond = dawn.condition_json.expect("dawn 应带条件");
    assert_eq!(dawn_cond["stat"].as_str(), Some("maxSkillCheckins"));
    assert_eq!(dawn_cond["value"].as_i64(), Some(7));

    let grand = row(&store, "grand");
    assert_eq!(grand.name, "百日专注");
    assert_eq!(
        grand.description.as_deref(),
        Some("任一技能累计记录 100 天")
    );

    // ② 单位写错的那条：「10,000 次」→「365 天」。统计量单位是天，一万天不是一万次
    let tenk = row(&store, "tenk");
    assert_eq!(tenk.name, "年度深耕");
    assert_eq!(tenk.description.as_deref(), Some("任一技能累计记录 365 天"));
    let tenk_cond = tenk.condition_json.expect("tenk 应带条件");
    assert_eq!(tenk_cond["stat"].as_str(), Some("maxSkillCheckins"));
    assert_eq!(
        tenk_cond["value"].as_i64(),
        Some(365),
        "阈值应改为 365 天，实际 {tenk_cond}"
    );
}

#[test]
fn rarity_follows_points_after_migration() {
    let db = temp_db("rarity");
    make_v6_skipped_db(&db);

    let store = open_at(&db);

    // 「完稿 150」原本是传说，却排在 160 / 180 的史诗之前 —— 越档，归一为史诗
    assert_eq!(row(&store, "done1").rarity.as_str(), "epic");
    // 200 / 300 仍是传说；10 仍是普通；40 仍是稀有
    assert_eq!(row(&store, "grand").rarity.as_str(), "legendary");
    assert_eq!(row(&store, "tenk").rarity.as_str(), "legendary");
    assert_eq!(row(&store, "first").rarity.as_str(), "common");
    assert_eq!(row(&store, "dawn").rarity.as_str(), "rare");
}

#[test]
fn migration_is_idempotent_on_an_already_correct_database() {
    let db = temp_db("idempotent");
    make_v6_skipped_db(&db);
    drop(open_at(&db)); // 第一次打开：V7 生效

    let store = open_at(&db); // 第二次打开：不应再动任何东西
    assert_eq!(row(&store, "tenk").name, "年度深耕");
    assert_eq!(row(&store, "done1").rarity.as_str(), "epic");
    assert_eq!(row(&store, "grand").name, "百日专注");
}
