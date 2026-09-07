//! Settings Repository（对应 TS `settings.ts`）：key-value，value 统一 TEXT。
//! JSON 类值走 get_json/set_json。

use rusqlite::{params, Connection};

use serde::de::DeserializeOwned;
use serde_json::Value;

use soloup_core::schema::SettingsRow;

use crate::errors::{StoreError, StoreErrorCode};

pub struct SettingsRepo<'a> {
    pub db: &'a Connection,
}

impl SettingsRepo<'_> {
    pub fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        let mut stmt = self.db.prepare("SELECT value FROM settings WHERE key = ?")?;
        let mut rows = stmt.query_map(params![key], |r| r.get::<_, String>(0))?;
        match rows.next() {
            Some(v) => Ok(Some(v?)),
            None => Ok(None),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.db.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// 读取并 JSON 解析；键不存在返回 Ok(None)；值损坏抛 ERR_SETTINGS_CORRUPT。
    pub fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StoreError> {
        match self.get(key)? {
            None => Ok(None),
            Some(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| StoreError::new(StoreErrorCode::CorruptSettings, format!("设置 {key} 的值不是合法 JSON：{e}"))),
        }
    }

    pub fn set_json(&self, key: &str, value: &Value) -> Result<(), StoreError> {
        self.set(key, &value.to_string())
    }

    pub fn list(&self) -> Result<Vec<SettingsRow>, StoreError> {
        let mut stmt = self.db.prepare("SELECT key, value FROM settings ORDER BY key ASC")?;
        let rows = stmt.query_map([], |r| {
            Ok(SettingsRow {
                key: r.get(0)?,
                value: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
