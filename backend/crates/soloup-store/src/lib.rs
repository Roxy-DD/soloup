//! soloup-store —— SQLite 持久化层（rusqlite），对应 TS `@soloup/store`。
//! 单文件 WAL 数据库 + PRAGMA user_version 增量迁移 + Repository 门面。
//! 事务边界由 `Store::tx` 显式控制（solver/service 层决定哪些操作原子）。

pub mod achievements;
pub mod attributes;
pub mod audit;
pub mod daily;
pub mod errors;
pub mod links;
pub mod migrations;
pub mod projects;
pub mod reset;
pub mod settings;
pub mod skills;

pub use errors::{StoreError, StoreErrorCode};
pub use migrations::latest_schema_version;

use std::env;
use std::path::Path;

use rusqlite::Connection;

use crate::achievements::AchievementRepo;
use crate::attributes::AttributeRepo;
use crate::audit::AuditRepo;
use crate::daily::DailyRepo;
use crate::links::LinkRepo;
use crate::projects::ProjectRepo;
use crate::reset::ResetRepo;
use crate::settings::SettingsRepo;
use crate::skills::SkillRepo;

/// 数据库路径解析：显式 path > `SOLOUP_DB_PATH` > `~/.soloup/soloup.db`；`:memory:` 内存库。
fn resolve_path(path: Option<&str>) -> String {
    if let Some(p) = path {
        return p.to_string();
    }
    if let Ok(p) = env::var("SOLOUP_DB_PATH") {
        return p.trim().to_string();
    }
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".soloup").join("soloup.db").to_string_lossy().to_string()
}

fn open_connection(path: &str) -> Result<Connection, StoreError> {
    if path != ":memory:" {
        if let Some(parent) = Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    let conn = Connection::open(path)?;
    if path != ":memory:" {
        let _mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    }
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), StoreError> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let latest = latest_schema_version();
    if current > latest {
        return Err(StoreError::new(
            StoreErrorCode::NewerSchema,
            format!("数据库 schema 版本 {current} 高于当前程序支持的 {latest}"),
        ));
    }
    for m in migrations::MIGRATIONS {
        if m.version > current {
            conn.execute_batch(m.sql)?;
            conn.pragma_update(None, "user_version", m.version)?;
        }
    }
    Ok(())
}

/// Store 门面：持有一个 SQLite 连接，对外暴露各 Repository。
pub struct Store {
    pub conn: Connection,
}

impl Store {
    /// 立即事务：闭包返回 Ok 则提交，否则回滚。错误类型由调用方决定（须可自 rusqlite::Error 构造）。
    pub fn tx<T, E>(&mut self, work: impl FnOnce(&Connection) -> Result<T, E>) -> Result<T, E>
    where
        E: From<rusqlite::Error>,
    {
        let tx = self.conn.transaction().map_err(E::from)?;
        match work(&tx) {
            Ok(v) => {
                tx.commit().map_err(E::from)?;
                Ok(v)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
    }

    pub fn attributes(&self) -> AttributeRepo<'_> {
        AttributeRepo { db: &self.conn }
    }
    pub fn skills(&self) -> SkillRepo<'_> {
        SkillRepo { db: &self.conn }
    }
    pub fn links(&self) -> LinkRepo<'_> {
        LinkRepo { db: &self.conn }
    }
    pub fn daily(&self) -> DailyRepo<'_> {
        DailyRepo { db: &self.conn }
    }
    pub fn projects(&self) -> ProjectRepo<'_> {
        ProjectRepo { db: &self.conn }
    }
    pub fn achievements(&self) -> AchievementRepo<'_> {
        AchievementRepo { db: &self.conn }
    }
    pub fn settings(&self) -> SettingsRepo<'_> {
        SettingsRepo { db: &self.conn }
    }
    pub fn audit(&self) -> AuditRepo<'_> {
        AuditRepo { db: &self.conn }
    }
    pub fn reset(&self) -> ResetRepo<'_> {
        ResetRepo { db: &self.conn }
    }
}

pub struct OpenStoreOptions {
    pub path: Option<String>,
}

impl Default for OpenStoreOptions {
    fn default() -> Self {
        Self { path: None }
    }
}

/// 打开（并自动迁移到最新 schema）数据库，返回 Store 门面。
pub fn open_store(options: OpenStoreOptions) -> Result<Store, StoreError> {
    let path = resolve_path(options.path.as_deref());
    let conn = open_connection(&path)?;
    migrate(&conn)?;
    Ok(Store { conn })
}

/// 生成实体 id（UUID v4，本地单机足够）。对应 TS store util generateId。
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
