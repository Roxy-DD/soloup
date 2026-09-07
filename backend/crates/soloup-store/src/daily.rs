//! DailyRecord Repository（对应 TS `daily.ts`）：每日打卡（record + 叶子技能成员）。
//! save 的「同日覆盖→清后插」须在调用方事务（Store::tx）内执行以保证原子。

use rusqlite::{params, Connection, OptionalExtension, Row};

use soloup_core::schema::{DailyRecord, DailyRecordSkill};

use crate::errors::StoreError;

pub struct DailySaveInput {
    pub project_id: Option<String>,
    pub note: Option<String>,
    pub skill_ids: Vec<String>,
}

pub struct DailyRepo<'a> {
    pub db: &'a Connection,
}

fn rec_from_row(row: &Row) -> rusqlite::Result<DailyRecord> {
    Ok(DailyRecord {
        date: row.get(0)?,
        project_id: row.get(1)?,
        note: row.get(2)?,
        settled: row.get(3)?,
    })
}

impl DailyRepo<'_> {
    pub fn get_record(&self, date: &str) -> Result<Option<DailyRecord>, StoreError> {
        let mut stmt = self
            .db
            .prepare("SELECT date, project_id, note, settled FROM daily_records WHERE date = ?1")?;
        let mut rows = stmt.query_map(params![date], rec_from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// [from, to] 闭区间（任一 None=不设界）；按日期升序。
    pub fn list_records(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<DailyRecord>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT date, project_id, note, settled FROM daily_records
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2) ORDER BY date ASC",
        )?;
        let rows = stmt.query_map(params![from, to], rec_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn skill_ids_on(&self, date: &str) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT skill_id FROM daily_record_skills WHERE date = ?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![date], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_members(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyRecordSkill>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT date, skill_id FROM daily_record_skills
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2) ORDER BY date ASC, rowid ASC",
        )?;
        let rows = stmt.query_map(params![from, to], |r| {
            Ok(DailyRecordSkill {
                date: r.get(0)?,
                skill_id: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 保存/覆盖某日打卡（含成员替换：清后插）。校验技能存在 + 去重。
    pub fn save(&self, date: &str, input: &DailySaveInput) -> Result<DailyRecord, StoreError> {
        // 校验技能存在 + 去重
        let mut skill_ids: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id in &input.skill_ids {
            if seen.insert(id.clone()) {
                let exists: Option<i64> = self
                    .db
                    .query_row("SELECT 1 FROM skills WHERE id = ?1", params![id], |r| r.get(0))
                    .optional()?;
                if exists.is_none() {
                    return Err(StoreError::not_found(format!("技能不存在：{id}")));
                }
                skill_ids.push(id.clone());
            }
        }
        self.db.execute(
            "INSERT INTO daily_records (date, project_id, note, settled) VALUES (?1, ?2, ?3, 0)
             ON CONFLICT(date) DO UPDATE SET project_id = excluded.project_id, note = excluded.note",
            params![date, input.project_id, input.note],
        )?;
        self.db
            .execute("DELETE FROM daily_record_skills WHERE date = ?1", params![date])?;
        for id in &skill_ids {
            self.db.execute(
                "INSERT INTO daily_record_skills (date, skill_id) VALUES (?1, ?2)",
                params![date, id],
            )?;
        }
        Ok(DailyRecord {
            date: date.to_string(),
            project_id: input.project_id.clone(),
            note: input.note.clone(),
            settled: 0,
        })
    }

    /// 清除某日打卡（成员经 ON DELETE CASCADE 一并清除），幂等。
    pub fn clear(&self, date: &str) -> Result<(), StoreError> {
        self.db
            .execute("DELETE FROM daily_records WHERE date = ?1", params![date])?;
        Ok(())
    }
}
