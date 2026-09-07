//! 引擎版演示种子（平移自 TS `apps/web/src/lib/bootstrap.ts`）：
//! 属性 6 / 技能树 15（8 叶子）/ 关联 8 / 84 天记录 / 项目 2 / 成就 8。
//! 落库事务内完成；首次启动或 `seed.demo` 时调用。

use soloup_core::dates::{add_days, today_iso};
use soloup_core::schema::{
    Achievement, AchievementType, Attribute, AttributeCategory, Difficulty, Project, ProjectStatus,
    Rarity, Skill, SkillCategory,
};
use soloup_store::Store;

pub const SEED_ATTRS: &[(&str, &str, &str, AttributeCategory, &str, i64)] = &[
    ("vit", "体质", "身体的力量、耐力与恢复", AttributeCategory::Physical, "#C6352B", 0),
    ("int", "智力", "思维、逻辑与学习深度", AttributeCategory::Mental, "#2456C4", 1),
    ("crea", "创造", "灵感、想象与产出", AttributeCategory::Creative, "#6D28D9", 2),
    ("will", "意志", "专注、坚持与抗压", AttributeCategory::Mental, "#B45309", 3),
    ("cha", "魅力", "表达、风格与影响力", AttributeCategory::Social, "#BE185D", 4),
    ("sen", "感知", "觉察、细腻与直觉", AttributeCategory::Mental, "#047857", 5),
];

pub struct SkillSeed {
    pub id: &'static str,
    pub name: &'static str,
    pub category: SkillCategory,
    pub difficulty: Option<Difficulty>,
    pub parent: Option<&'static str>,
}

pub const SEED_SKILLS: &[SkillSeed] = &[
    SkillSeed { id: "art", name: "艺术", category: SkillCategory::Cognitive, difficulty: None, parent: None },
    SkillSeed { id: "art-paint", name: "绘画", category: SkillCategory::Cognitive, difficulty: None, parent: Some("art") },
    SkillSeed { id: "art-music", name: "音乐", category: SkillCategory::Cognitive, difficulty: None, parent: Some("art") },
    SkillSeed { id: "sketch", name: "素描", category: SkillCategory::Cognitive, difficulty: Some(Difficulty::Normal), parent: Some("art-paint") },
    SkillSeed { id: "oil", name: "油画", category: SkillCategory::Cognitive, difficulty: Some(Difficulty::Normal), parent: Some("art-paint") },
    SkillSeed { id: "digi", name: "数字绘画", category: SkillCategory::Cognitive, difficulty: Some(Difficulty::Normal), parent: Some("art-paint") },
    SkillSeed { id: "guitar", name: "吉他", category: SkillCategory::Cognitive, difficulty: Some(Difficulty::Casual), parent: Some("art-music") },
    SkillSeed { id: "tech", name: "技术", category: SkillCategory::Knowledge, difficulty: None, parent: None },
    SkillSeed { id: "tech-code", name: "编程", category: SkillCategory::Knowledge, difficulty: None, parent: Some("tech") },
    SkillSeed { id: "fe", name: "前端开发", category: SkillCategory::Knowledge, difficulty: Some(Difficulty::Normal), parent: Some("tech-code") },
    SkillSeed { id: "rust", name: "Rust", category: SkillCategory::Knowledge, difficulty: Some(Difficulty::Hard), parent: Some("tech-code") },
    SkillSeed { id: "body", name: "身体", category: SkillCategory::Physical, difficulty: None, parent: None },
    SkillSeed { id: "body-sport", name: "运动", category: SkillCategory::Physical, difficulty: None, parent: Some("body") },
    SkillSeed { id: "run", name: "跑步", category: SkillCategory::Physical, difficulty: Some(Difficulty::Normal), parent: Some("body-sport") },
    SkillSeed { id: "gym", name: "力量训练", category: SkillCategory::Physical, difficulty: Some(Difficulty::Hard), parent: Some("body-sport") },
];

/// 叶子 → 属性关联权重。
pub const SEED_LINKS: &[(&str, &[(&str, f64)])] = &[
    ("sketch", &[("crea", 0.5), ("sen", 0.5)]),
    ("oil", &[("crea", 0.6), ("sen", 0.4)]),
    ("digi", &[("crea", 0.7), ("int", 0.3)]),
    ("guitar", &[("crea", 0.5), ("cha", 0.5)]),
    ("fe", &[("int", 0.6), ("crea", 0.4)]),
    ("rust", &[("int", 0.8), ("will", 0.2)]),
    ("run", &[("vit", 0.7), ("will", 0.3)]),
    ("gym", &[("vit", 0.8), ("will", 0.2)]),
];

const CYCLE: [i64; 14] = [2, 0, 1, 3, 0, 2, 4, 1, 2, 0, 3, 1, 0, 2];
const LEAVES: [&str; 8] = ["sketch", "oil", "digi", "guitar", "fe", "rust", "run", "gym"];

/// 84 天确定性伪随机打卡记录（与 TS 一致的 CYCLE/LEAVES 取模）。
pub fn seed_records() -> Vec<(String, Vec<String>)> {
    let today = today_iso();
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for i in (1..=84).rev() {
        let lit = CYCLE[i as usize % CYCLE.len()];
        if lit == 0 {
            continue;
        }
        let n = (lit - 1).clamp(1, 3);
        let mut ids: Vec<String> = Vec::new();
        for k in 0..n {
            let s = LEAVES[((i * 5 + k * 3) as usize) % LEAVES.len()];
            if !ids.iter().any(|x| x == s) {
                ids.push(s.to_string());
            }
        }
        let date = add_days(&today, -i);
        seen.push(date.clone());
        out.push((date, ids));
    }
    // 补齐近 7 天，保证 streak
    for i in 1..=7 {
        let date = add_days(&today, -i);
        if !seen.iter().any(|d| *d == date) {
            out.push((date, vec!["fe".to_string()]));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub const SEED_PROJECTS: &[(&str, &str, &str, &str, Option<&str>, ProjectStatus)] = &[
    (
        "p-thesis",
        "毕业设计 · 萌趣记账小程序",
        "从开题到论文的完整冲刺：包含架构定型、AI 记账模块联调与论文打磨。",
        "2026-03-01",
        None,
        ProjectStatus::Active,
    ),
    (
        "p-ptz",
        "PTZ 目标跟踪系统调研",
        "跟踪算法选型综述与低成本方案验证（已完成，历史记录冻结在收尾日）。",
        "2025-10-01",
        Some("2025-12-20"),
        ProjectStatus::Completed,
    ),
];

pub fn seed_achievements() -> Vec<Achievement> {
    let defs: &[(&str, &str, &str, AchievementType, Rarity, i64, bool, bool, &[&str], Option<f64>)] = &[
        // id, name, desc, type, rarity, points, legendary, hidden, requires, reveal_at
        ("first",  "初次觉醒", "完成第一次每日记录",       AchievementType::Milestone, Rarity::Common,    10, false, true,  &[],        Some(0.5)),
        ("twin",   "双线并进", "单日点亮 2 项以上属性",   AchievementType::Attribute, Rarity::Common,    20, false, true,  &["first"],  Some(0.5)),
        ("week7",  "七日之约", "连续记录 7 天不间断",     AchievementType::Milestone, Rarity::Rare,      30, false, false, &[],        None),
        ("dawn",   "破晓之光", "任一技能达到 4 级",       AchievementType::Skill,     Rarity::Rare,      40, false, false, &[],        None),
        ("d100",   "百日筑基", "累计记录 100 天",         AchievementType::Milestone, Rarity::Epic,     100, false, false, &[],        None),
        ("allsix", "六艺俱全", "单日点亮全部属性",       AchievementType::Attribute, Rarity::Epic,     120, false, true,  &["twin"],   Some(0.4)),
        ("done1",  "完稿",     "完成第一个项目",          AchievementType::Project,   Rarity::Legendary, 150, true,  false, &["week7","dawn"], None),
        ("grand",  "宗师之路", "任一技能达到 20 级",      AchievementType::Skill,     Rarity::Legendary, 200, true,  false, &["dawn"],   None),
        ("tenk",   "万时之功", "某个技能累计打卡 10,000 天", AchievementType::Skill,  Rarity::Legendary, 300, true,  false, &["d100"],   None),
    ];
    defs.iter()
        .map(|(id, name, desc, t, r, points, legendary, hidden, requires, reveal_at)| {
            let mut cond = serde_json::Map::new();
            if *legendary { cond.insert("legendary".into(), serde_json::json!(true)); }
            Achievement {
                id: (*id).to_string(),
                name: (*name).to_string(),
                description: Some((*desc).to_string()),
                condition_json: if cond.is_empty() { None } else { Some(serde_json::Value::Object(cond)) },
                r#type: *t,
                rarity: *r,
                points: *points,
                unlocked_at: None,
                hidden: *hidden,
                requires: requires.iter().map(|s| s.to_string()).collect(),
                reveal_at: *reveal_at,
            }
        })
        .collect()
}

/// 清空并写入演示种子（事务内）。
pub fn seed_demo(store: &mut Store) -> Result<(), soloup_store::StoreError> {
    store.reset().wipe(&[])?;
    let created_at = seed_records().first().map(|(d, _)| d.clone()).unwrap_or_else(today_iso);

    store.tx(|c| -> Result<(), soloup_store::StoreError> {
        let attrs = soloup_store::attributes::AttributeRepo { db: c };
        for (id, name, desc, cat, color, sort) in SEED_ATTRS {
            attrs.create(&Attribute {
                id: (*id).to_string(),
                name: (*name).to_string(),
                description: Some((*desc).to_string()),
                base_value: 0.0,
                max_value: 100.0,
                alpha: 1.2,
                w0: 400.0,
                category: Some(*cat),
                color: Some((*color).to_string()),
                icon: None,
                sort: *sort,
            })?;
        }

        let skills = soloup_store::skills::SkillRepo { db: c };
        for s in SEED_SKILLS {
            skills.create(&Skill {
                id: (*s.id).to_string(),
                name: (*s.name).to_string(),
                description: None,
                parent_id: s.parent.map(|p| p.to_string()),
                category: s.category,
                difficulty: s.difficulty.unwrap_or(Difficulty::Normal),
                curve_type: None,
                c: 0.0,
                v: 0.0,
                last_settled_date: None,
                created_at: created_at.clone(),
                archived_at: None,
                color: None,
                icon: None,
                sort: 0,
            })?;
        }

        let links = soloup_store::links::LinkRepo { db: c };
        for (leaf, ls) in SEED_LINKS {
            let inputs: Vec<soloup_store::links::SkillLinkInput> = ls
                .iter()
                .map(|(a, w)| soloup_store::links::SkillLinkInput {
                    attribute_id: (*a).to_string(),
                    weight: *w,
                })
                .collect();
            links.set_for_skill(leaf, &inputs)?;
        }

        let daily = soloup_store::daily::DailyRepo { db: c };
        for (date, ids) in seed_records() {
            daily.save(
                &date,
                &soloup_store::daily::DailySaveInput {
                    project_id: None,
                    note: None,
                    skill_ids: ids,
                },
            )?;
        }

        let projects = soloup_store::projects::ProjectRepo { db: c };
        for (id, name, desc, start, end, status) in SEED_PROJECTS {
            projects.create(&Project {
                id: (*id).to_string(),
                name: (*name).to_string(),
                description: Some((*desc).to_string()),
                start_date: (*start).to_string(),
                end_date: end.map(|e| e.to_string()),
                status: *status,
                color: None,
            })?;
        }

        let achs = soloup_store::achievements::AchievementRepo { db: c };
        for a in seed_achievements() {
            achs.create(&a)?;
        }

        soloup_store::settings::SettingsRepo { db: c }.set("seeded", "1")?;
        soloup_store::settings::SettingsRepo { db: c }.set_json("life_axis", &serde_json::json!({ "born": "2002-05-20", "expectancy": 120 }))?;
        Ok(())
    })
}
