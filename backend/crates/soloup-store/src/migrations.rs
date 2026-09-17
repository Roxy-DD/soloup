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

pub const MIGRATIONS: &[Migration] = &[
    V1_INIT,
    V2_ACHIEVEMENT_REVEAL,
    V3_SKILL_IS_BRANCH,
    V4_ACHIEVEMENT_CONDITIONS,
    V5_ACHIEVEMENT_REVEAL_FIELDS,
];

pub fn latest_schema_version() -> i64 {
    MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
}
