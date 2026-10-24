//! SQLite 迁移注册表 —— PRAGMA user_version 记录已应用版本（对应 TS `migrations.ts`）。
//! 新增版本只能向后追加。

pub struct Migration {
    pub version: i64,
    pub label: &'static str,
    pub sql: &'static str,
}

pub const V1_INIT: Migration = Migration {
    version: 1,
    label: "init：§4.3 全部核心表（v1 主表形态）",
    sql: r#"
CREATE TABLE IF NOT EXISTS attributes (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  base_value  REAL NOT NULL DEFAULT 0,
  max_value   REAL NOT NULL DEFAULT 100,
  alpha       REAL NOT NULL DEFAULT 1.2,
  w0          REAL NOT NULL DEFAULT 400,
  category    TEXT,
  color       TEXT,
  icon        TEXT,
  sort        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS skills (
  id                TEXT PRIMARY KEY,
  name              TEXT NOT NULL,
  description       TEXT,
  parent_id         TEXT REFERENCES skills(id) ON DELETE RESTRICT,
  category          TEXT NOT NULL,
  difficulty        TEXT NOT NULL,
  curve_type        TEXT,
  c                 REAL NOT NULL DEFAULT 0,
  v                 REAL NOT NULL DEFAULT 0,
  last_settled_date TEXT,
  created_at        TEXT NOT NULL,
  archived_at       TEXT,
  color             TEXT,
  icon              TEXT,
  sort              INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS skill_attributes (
  skill_id     TEXT NOT NULL REFERENCES skills(id)     ON DELETE CASCADE,
  attribute_id TEXT NOT NULL REFERENCES attributes(id) ON DELETE CASCADE,
  weight       REAL NOT NULL,
  PRIMARY KEY (skill_id, attribute_id)
);

CREATE TABLE IF NOT EXISTS daily_records (
  date       TEXT PRIMARY KEY,
  project_id TEXT,
  note       TEXT,
  settled    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS daily_record_skills (
  date     TEXT NOT NULL REFERENCES daily_records(date) ON DELETE CASCADE,
  skill_id TEXT NOT NULL REFERENCES skills(id)          ON DELETE CASCADE,
  PRIMARY KEY (date, skill_id)
);

CREATE TABLE IF NOT EXISTS projects (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  start_date  TEXT NOT NULL,
  end_date    TEXT,
  status      TEXT NOT NULL,
  color       TEXT
);

CREATE TABLE IF NOT EXISTS achievements (
  id             TEXT PRIMARY KEY,
  name           TEXT NOT NULL,
  description    TEXT,
  condition_json TEXT,
  type           TEXT NOT NULL,
  rarity         TEXT NOT NULL,
  points         INTEGER NOT NULL DEFAULT 0,
  unlocked_at    TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  at          TEXT NOT NULL,
  actor       TEXT NOT NULL,
  tool        TEXT,
  params_json TEXT
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
"#,
};

pub const V2_ACHIEVEMENT_REVEAL: Migration = Migration {
    version: 2,
    label: "成就渐进揭示：hidden + requires + reveal_at",
    sql: r#"
ALTER TABLE achievements ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE achievements ADD COLUMN requires_json TEXT;
ALTER TABLE achievements ADD COLUMN reveal_at REAL;
"#,
};

pub const V3_SKILL_IS_BRANCH: Migration = Migration {
    version: 3,
    label: "技能分支标记：is_branch",
    sql: r#"
ALTER TABLE skills ADD COLUMN is_branch INTEGER NOT NULL DEFAULT 0;
"#,
};

/// 内置成就的条件数据化。
/// ① 补齐缺失的内置成就（早期版本的种子只播了 8 个，且 condition_json 存的是
///    `{"legendary":true}` 这类无效值，判定因此只能硬编码在 soloup-server 里）；
/// ② 把阈值统一写成条件 DSL。
/// 只碰这 9 个内置 id，用户自定义成就不受影响。新库由 seed.rs 直接写入，此迁移对空表无副作用。
pub const V4_ACHIEVEMENT_CONDITIONS: Migration = Migration {
    version: 4,
    label: "成就条件数据化：补齐内置成就并写入 stat/operator/value",
    sql: r#"
INSERT OR IGNORE INTO achievements
  (id, name, description, condition_json, type, rarity, points, unlocked_at, hidden, requires_json, reveal_at) VALUES
  ('first',  '初次觉醒', '完成第一次每日记录',         '{"stat":"totalDays","operator":">=","value":1}',            'milestone', 'common',    10,  NULL, 1, NULL, 0.5),
  ('twin',   '双线并进', '单日点亮 2 项以上属性',      '{"stat":"maxLitOneDay","operator":">=","value":2}',         'attribute', 'common',    20,  NULL, 1, NULL, 0.5),
  ('week7',  '七日之约', '连续记录 7 天不间断',        '{"stat":"streak","operator":">=","value":7}',               'milestone', 'rare',      30,  NULL, 0, NULL, NULL),
  ('dawn',   '破晓之光', '任一技能达到 4 级',          '{"stat":"maxSkillLv","operator":">=","value":4}',           'skill',     'rare',      40,  NULL, 0, NULL, NULL),
  ('d100',   '百日筑基', '累计记录 100 天',            '{"stat":"totalDays","operator":">=","value":100}',          'milestone', 'epic',     100,  NULL, 0, NULL, NULL),
  ('allsix', '六艺俱全', '单日点亮全部属性',           '{"stat":"maxLitOneDay","operator":">=","ref":"totalAttrs"}','attribute', 'epic',     120,  NULL, 1, NULL, 0.4),
  ('done1',  '完稿',     '完成第一个项目',             '{"stat":"projectsCompleted","operator":">=","value":1}',    'project',   'legendary',150,  NULL, 0, NULL, NULL),
  ('grand',  '宗师之路', '任一技能达到 20 级',         '{"stat":"maxSkillLv","operator":">=","value":20}',          'skill',     'legendary',200,  NULL, 0, NULL, NULL),
  ('tenk',   '万时之功', '某个技能累计打卡 10,000 天', '{"stat":"maxSkillCheckins","operator":">=","value":10000}', 'skill',     'legendary',300,  NULL, 0, NULL, NULL);

UPDATE achievements SET condition_json = '{"stat":"totalDays","operator":">=","value":1}'             WHERE id = 'first';
UPDATE achievements SET condition_json = '{"stat":"maxLitOneDay","operator":">=","value":2}'          WHERE id = 'twin';
UPDATE achievements SET condition_json = '{"stat":"streak","operator":">=","value":7}'                WHERE id = 'week7';
UPDATE achievements SET condition_json = '{"stat":"maxSkillLv","operator":">=","value":4}'            WHERE id = 'dawn';
UPDATE achievements SET condition_json = '{"stat":"totalDays","operator":">=","value":100}'           WHERE id = 'd100';
UPDATE achievements SET condition_json = '{"stat":"maxLitOneDay","operator":">=","ref":"totalAttrs"}' WHERE id = 'allsix';
UPDATE achievements SET condition_json = '{"stat":"projectsCompleted","operator":">=","value":1}'     WHERE id = 'done1';
UPDATE achievements SET condition_json = '{"stat":"maxSkillLv","operator":">=","value":20}'           WHERE id = 'grand';
UPDATE achievements SET condition_json = '{"stat":"maxSkillCheckins","operator":">=","value":10000}'  WHERE id = 'tenk';
"#,
};

/// 内置成就的「渐进揭示」字段补齐。
/// V4 只覆写了 condition_json；早期种子写下的 9 行里 hidden / requires_json / reveal_at
/// 仍停在 V2 加列时的默认值（0 / NULL / NULL），seed.rs 里的设计值从未落到老库上。
/// 只碰这 9 个内置 id，用户自定义成就不受影响；新库由 seed.rs 直接写入，此迁移对空表无副作用。
pub const V5_ACHIEVEMENT_REVEAL_FIELDS: Migration = Migration {
    version: 5,
    label: "成就揭示字段数据化：hidden / requires_json / reveal_at",
    sql: r#"
UPDATE achievements SET hidden = 1, requires_json = '[]',               reveal_at = 0.5  WHERE id = 'first';
UPDATE achievements SET hidden = 1, requires_json = '["first"]',        reveal_at = 0.5  WHERE id = 'twin';
UPDATE achievements SET hidden = 0, requires_json = '[]',               reveal_at = NULL WHERE id = 'week7';
UPDATE achievements SET hidden = 0, requires_json = '[]',               reveal_at = NULL WHERE id = 'dawn';
UPDATE achievements SET hidden = 0, requires_json = '[]',               reveal_at = NULL WHERE id = 'd100';
UPDATE achievements SET hidden = 1, requires_json = '["twin"]',         reveal_at = 0.4  WHERE id = 'allsix';
UPDATE achievements SET hidden = 0, requires_json = '["week7","dawn"]', reveal_at = NULL WHERE id = 'done1';
UPDATE achievements SET hidden = 0, requires_json = '["dawn"]',         reveal_at = NULL WHERE id = 'grand';
UPDATE achievements SET hidden = 0, requires_json = '["d100"]',         reveal_at = NULL WHERE id = 'tenk';
"#,
};

/// 内置技能成就改用真实可观测的按技能打卡天数，不再把未经验证的等级曲线当作能力测量。
/// 兼容升级已有数据库；仅更新内置成就定义，不修改用户自定义成就及打卡数据。
pub const V6_OBSERVABLE_SKILL_ACHIEVEMENTS: Migration = Migration {
    version: 6,
    label: "技能成就改用可观测的练习记录里程碑",
    sql: r#"
UPDATE achievements SET
  description=CASE WHEN description='任一技能达到 4 级' THEN '任一技能累计记录 7 天' ELSE description END,
  condition_json='{"stat":"maxSkillCheckins","operator":">=","value":7}'
  WHERE id='dawn' AND condition_json='{"stat":"maxSkillLv","operator":">=","value":4}';
UPDATE achievements SET name=CASE WHEN name='宗师之路' THEN '百日专注' ELSE name END,
  description=CASE WHEN description='任一技能达到 20 级' THEN '任一技能累计记录 100 天' ELSE description END,
  condition_json='{"stat":"maxSkillCheckins","operator":">=","value":100}'
  WHERE id='grand' AND condition_json='{"stat":"maxSkillLv","operator":">=","value":20}';
UPDATE achievements SET name=CASE WHEN name='万时之功' THEN '年度深耕' ELSE name END,
  description=CASE WHEN description='某个技能累计打卡 10,000 天' THEN '任一技能累计记录 365 天' ELSE description END,
  condition_json='{"stat":"maxSkillCheckins","operator":">=","value":365}'
  WHERE id='tenk' AND condition_json='{"stat":"maxSkillCheckins","operator":">=","value":10000}';
"#,
};

/// V6 是**空跑**的：它的 WHERE 用序列化后的 JSON 字符串做等值比较，而库里存的键顺序是
/// `operator/stat/value`、V6 写的却是 `stat/operator/value` —— 字符串不等，于是
/// `dawn` / `grand` / `tenk` 三条一条都没被改到，`user_version` 却照样跳到 6。
/// 新装用户走 seed 拿到的是新文案，老库停在旧文案，两种库永久分叉。
///
/// 这一版改用 `json_extract` 按字段匹配（键顺序无关），并且：
/// ① 幂等 —— 已经是对的库再跑一遍不动它；
/// ② 兼容「V6 生效过」与「V6 空跑过」两种历史；
/// ③ 顺手把稀有度归到分值的单调分档（见 `Rarity::from_points`）。
///    真实库 22 条里只有「完稿 150」越档（金卡却低于两张紫卡），归一后金卡 5 → 4 张。
pub const V7_ACHIEVEMENT_CONDITION_JSON_MATCH: Migration = Migration {
    version: 7,
    label: "补齐 V6 空跑：成就条件按 JSON 字段匹配，稀有度归一到分值分档",
    sql: r#"
UPDATE achievements SET
  description = '任一技能累计记录 7 天',
  condition_json = '{"stat":"maxSkillCheckins","operator":">=","value":7}'
  WHERE json_extract(condition_json,'$.stat')='maxSkillLv'
    AND json_extract(condition_json,'$.value')=4;

UPDATE achievements SET
  name = '百日专注',
  description = '任一技能累计记录 100 天',
  condition_json = '{"stat":"maxSkillCheckins","operator":">=","value":100}'
  WHERE json_extract(condition_json,'$.stat')='maxSkillLv'
    AND json_extract(condition_json,'$.value')=20;

-- 旧文案有两种写法（「累计使用 10,000 次」是错单位的那个），匹配条件是 stat+value，
-- 所以描述直接覆盖：这三行的旧描述本来就是在描述旧条件。
UPDATE achievements SET
  name = '年度深耕',
  description = '任一技能累计记录 365 天',
  condition_json = '{"stat":"maxSkillCheckins","operator":">=","value":365}'
  WHERE json_extract(condition_json,'$.stat')='maxSkillCheckins'
    AND json_extract(condition_json,'$.value')=10000;

UPDATE achievements SET rarity = CASE
    WHEN points <= 20 THEN 'common'
    WHEN points <= 60 THEN 'rare'
    WHEN points < 200 THEN 'epic'
    ELSE 'legendary' END
  WHERE points IS NOT NULL
    AND rarity <> CASE
    WHEN points <= 20 THEN 'common'
    WHEN points <= 60 THEN 'rare'
    WHEN points < 200 THEN 'epic'
    ELSE 'legendary' END;
"#,
};

pub const MIGRATIONS: &[Migration] = &[
    V1_INIT,
    V2_ACHIEVEMENT_REVEAL,
    V3_SKILL_IS_BRANCH,
    V4_ACHIEVEMENT_CONDITIONS,
    V5_ACHIEVEMENT_REVEAL_FIELDS,
    V6_OBSERVABLE_SKILL_ACHIEVEMENTS,
    V7_ACHIEVEMENT_CONDITION_JSON_MATCH,
];

pub fn latest_schema_version() -> i64 {
    MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
}
