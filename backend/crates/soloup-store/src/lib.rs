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

/// 迁移前自动备份：保留最近多少份快照（按文件名即时间排序，超出的从最旧开始删）。
const BACKUP_KEEP: usize = 10;
/// 逃生舱：`SOLOUP_SKIP_BACKUP=1` 时跳过迁移前备份（备份目录不可写时不至于卡死启动）。
const SKIP_BACKUP_ENV: &str = "SOLOUP_SKIP_BACKUP";

/// SQLite 字符串字面量转义（单引号写成两个单引号）。
fn sql_quote(s: &str) -> String {
    s.replace('\'', "''")
}

/// 备份目录 = 数据库文件同级的 `backups/`；内存库没有备份可言。
fn backup_dir_of(db_path: &str) -> Option<std::path::PathBuf> {
    if db_path == ":memory:" {
        return None;
    }
    let parent = Path::new(db_path).parent().filter(|p| !p.as_os_str().is_empty())?;
    Some(parent.join("backups"))
}

/// 库里是否已经有业务表（用于判断「空库不值得备份」）。
fn has_user_tables(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
         AND name IN ('daily_records','skills','attributes','projects')",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// 迁移前的一致性快照。
///
/// 这里**不能**用 `std::fs::copy`：库跑在 WAL 模式下，主库文件里可能缺着尚未 checkpoint
/// 的写入（本机实测主库停在 9/10、`-wal` 里还压着 98KB），裸复制会丢掉最近的数据。
/// `VACUUM INTO` 交给 SQLite 自己导出，拿到的是完整、一致、且已合并 WAL 的副本。
fn backup_before_migrate(
    conn: &Connection,
    db_path: &str,
    from_version: i64,
) -> Result<Option<std::path::PathBuf>, StoreError> {
    let skip = env::var(SKIP_BACKUP_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip {
        return Ok(None);
    }
    let Some(dir) = backup_dir_of(db_path) else { return Ok(None) };
    let hint = format!("（可用 {SKIP_BACKUP_ENV}=1 跳过备份）");

    std::fs::create_dir_all(&dir).map_err(|e| {
        StoreError::new(
            StoreErrorCode::BackupFailed,
            format!("无法创建备份目录 {}：{e} {hint}", dir.display()),
        )
    })?;

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let file = dir.join(format!("soloup-{stamp}-v{from_version}.db"));
    if file.exists() {
        // 同一秒内重复触发：复用已有快照，不覆盖。
        return Ok(Some(file));
    }

    conn.execute_batch(&format!("VACUUM INTO '{}'", sql_quote(&file.to_string_lossy())))
        .map_err(|e| {
            StoreError::new(
                StoreErrorCode::BackupFailed,
                format!("迁移前备份失败 {}：{e} {hint}", file.display()),
            )
        })?;
    prune_backups(&dir);
    Ok(Some(file))
}

/// 只保留最近 `BACKUP_KEEP` 份快照。只认本程序自己产出的命名（`soloup-<时间戳>-v<n>.db`），
/// 目录里其他文件一律不碰。
fn prune_backups(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut ours: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("soloup-") && n.ends_with(".db"))
        })
        .collect();
    if ours.len() <= BACKUP_KEEP {
        return;
    }
    ours.sort();
    for old in &ours[..ours.len() - BACKUP_KEEP] {
        let _ = std::fs::remove_file(old);
    }
}

fn migrate(conn: &Connection, db_path: &str) -> Result<(), StoreError> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let latest = latest_schema_version();
    if current > latest {
        return Err(StoreError::new(
            StoreErrorCode::NewerSchema,
            format!("数据库 schema 版本 {current} 高于当前程序支持的 {latest}"),
        ));
    }
    if current < latest && (current > 0 || has_user_tables(conn)) {
        if let Some(snapshot) = backup_before_migrate(conn, db_path, current)? {
            println!("[soloup] 迁移前已备份 v{current} → {}", snapshot.display());
        }
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
    migrate(&conn, &path)?;
    Ok(Store { conn })
}

/// 生成实体 id（UUID v4，本地单机足够）。对应 TS store util generateId。
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
