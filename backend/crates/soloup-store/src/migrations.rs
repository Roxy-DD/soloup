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

pub const MIGRATIONS: &[Migration] = &[V1_INIT, V2_ACHIEVEMENT_REVEAL];

pub fn latest_schema_version() -> i64 {
    MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
}
