//! Achievement Repository（对应 TS `achievements.ts`）。condition_json 序列化为 JSON 文本列。

use rusqlite::{params, Connection, Row};
use serde_json::Value;

use soloup_core::schema::{Achievement, AchievementType, DbEnum, Rarity};

use crate::errors::StoreError;
use crate::generate_id;

pub struct AchievementRepo<'a> {
    pub db: &'a Connection,
}

fn from_row(row: &Row) -> rusqlite::Result<Achievement> {
    let r#type: String = row.get(4)?;
    let rarity: String = row.get(5)?;
    let cond_text: Option<String> = row.get(3)?;
    let hidden_i: i32 = row.get(8)?;
    let req_text: Option<String> = row.get(9)?;
    Ok(Achievement {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        condition_json: cond_text.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        r#type: AchievementType::parse(&r#type).unwrap_or(AchievementType::Milestone),
        rarity: Rarity::parse(&rarity).unwrap_or(Rarity::Common),
        points: row.get(6)?,
        unlocked_at: row.get(7)?,
        hidden: hidden_i != 0,
        requires: req_text.as_deref().and_then(|s| serde_json::from_str::<Vec<String>>(s).ok()).unwrap_or_default(),
        reveal_at: row.get(10)?,
    })
}

impl AchievementRepo<'_> {
    pub fn list(&self) -> Result<Vec<Achievement>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, description, condition_json, type, rarity, points, unlocked_at, hidden, requires_json, reveal_at FROM achievements ORDER BY type ASC, rarity ASC, name ASC",
        )?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, id: &str) -> Result<Option<Achievement>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, description, condition_json, type, rarity, points, unlocked_at, hidden, requires_json, reveal_at FROM achievements WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    fn cond_text(v: &Option<Value>) -> Option<String> {
        v.as_ref().map(|x| x.to_string())
    }

    pub fn create(&self, a: &Achievement) -> Result<Achievement, StoreError> {
        let mut row = a.clone();
        if row.id.is_empty() {
            row.id = generate_id();
        }
        let req_json = serde_json::to_string(&row.requires).unwrap_or_else(|_| "[]".into());
        self.db.execute(
            "INSERT INTO achievements (id, name, description, condition_json, type, rarity, points, unlocked_at, hidden, requires_json, reveal_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                row.id,
                row.name,
                row.description,
                Self::cond_text(&row.condition_json),
                row.r#type.as_str(),
                row.rarity.as_str(),
                row.points,
                row.unlocked_at,
                row.hidden as i32,
                req_json,
                row.reveal_at,
            ],
        )?;
        Ok(row)
    }

    pub fn update(&self, a: &Achievement) -> Result<Achievement, StoreError> {
        let req_json = serde_json::to_string(&a.requires).unwrap_or_else(|_| "[]".into());
        let n = self.db.execute(
            "UPDATE achievements SET name=?2, description=?3, condition_json=?4, type=?5, rarity=?6, points=?7, unlocked_at=?8, hidden=?9, requires_json=?10, reveal_at=?11 WHERE id=?1",
            params![
                a.id,
                a.name,
                a.description,
                Self::cond_text(&a.condition_json),
                a.r#type.as_str(),
                a.rarity.as_str(),
                a.points,
                a.unlocked_at,
                a.hidden as i32,
                req_json,
                a.reveal_at,
            ],
        )?;
        if n == 0 {
            return Err(StoreError::not_found(format!("成就不存在：{}", a.id)));
        }
        Ok(a.clone())
    }

    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.db.execute("DELETE FROM achievements WHERE id = ?1", params![id])?;
        Ok(())
    }
}
