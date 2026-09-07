//! Reset Repository（对应 TS `reset.ts`）：清空领域数据（种子重灌/演示初始化专用）。
//! 删除时临时关闭 FK（整体清空无需逐行约束），事务内全删并兜底恢复 ON。

use rusqlite::{params, Connection};

use crate::errors::StoreError;

const DOMAIN_TABLES: [&str; 8] = [
    "daily_record_skills",
    "daily_records",
    "skill_attributes",
    "skills",
    "attributes",
    "projects",
    "achievements",
    "audit_log",
];

pub struct ResetRepo<'a> {
    pub db: &'a Connection,
}

impl ResetRepo<'_> {
    /// 清空全部领域表 + settings（可保留若干 settings key），幂等。
    pub fn wipe(&self, keep_settings: &[&str]) -> Result<(), StoreError> {
        self.db.pragma_update(None, "foreign_keys", "OFF")?;
        let result = (|| -> Result<(), StoreError> {
            for t in DOMAIN_TABLES {
                self.db.execute(&format!("DELETE FROM {t}"), [])?;
            }
            if keep_settings.is_empty() {
                self.db.execute("DELETE FROM settings", [])?;
            } else {
                let keys: Vec<String> = {
                    let mut stmt = self.db.prepare("SELECT key FROM settings")?;
                    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                    rows.collect::<rusqlite::Result<Vec<_>>>()?
                };
                for k in keys {
                    if !keep_settings.contains(&k.as_str()) {
                        self.db.execute("DELETE FROM settings WHERE key = ?1", params![k])?;
                    }
                }
            }
            Ok(())
        })();
        self.db.pragma_update(None, "foreign_keys", "ON")?;
        result
    }
}
