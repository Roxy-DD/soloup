//! Skill Repository：CRUD + 树不变量（环检测）+ 归档/恢复 + 结算专用通道。
//! 对应 TS `skills.ts`。C/V/last_settled_date 的唯一写入口是 settle_accounts。

use rusqlite::{params, Connection, Row};

use soloup_core::dates::today_iso;
use soloup_core::schema::{CurveType, DbEnum, Difficulty, Skill, SkillCategory};

use crate::errors::{StoreError, StoreErrorCode};
use crate::generate_id;

pub const COLS: &str = "id, name, description, parent_id, category, difficulty, curve_type, c, v, last_settled_date, created_at, archived_at, color, icon, sort, is_branch";

fn from_row(row: &Row) -> rusqlite::Result<Skill> {
    let category: String = row.get(4)?;
    let difficulty: String = row.get(5)?;
    let curve_type: Option<String> = row.get(6)?;
    Ok(Skill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        parent_id: row.get(3)?,
        category: SkillCategory::parse(&category).unwrap_or(SkillCategory::Knowledge),
        difficulty: Difficulty::parse(&difficulty).unwrap_or(Difficulty::Normal),
        curve_type: curve_type
            .as_deref()
            .and_then(CurveType::parse),
        c: row.get(7)?,
        v: row.get(8)?,
        last_settled_date: row.get(9)?,
        created_at: row.get(10)?,
        archived_at: row.get(11)?,
        color: row.get(12)?,
        icon: row.get(13)?,
        sort: row.get(14)?,
        is_branch: row.get::<_, bool>(15).unwrap_or(false),
    })
}

pub struct SkillRepo<'a> {
    pub db: &'a Connection,
}

fn subtree_sql(id: &str, db: &Connection) -> Result<Vec<String>, StoreError> {
    let mut stmt = db.prepare(
        "WITH RECURSIVE sub(id) AS (
           SELECT id FROM skills WHERE id = ?1
           UNION ALL
           SELECT s.id FROM skills s JOIN sub ON s.parent_id = sub.id
         ) SELECT id FROM sub",
    )?;
    let rows = stmt.query_map(params![id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

impl SkillRepo<'_> {
    pub fn list(&self) -> Result<Vec<Skill>, StoreError> {
        let sql = format!("SELECT {COLS} FROM skills WHERE archived_at IS NULL ORDER BY sort ASC, created_at ASC, name ASC");
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_all(&self) -> Result<Vec<Skill>, StoreError> {
        let sql = format!("SELECT {COLS} FROM skills ORDER BY sort ASC, created_at ASC, name ASC");
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, id: &str) -> Result<Option<Skill>, StoreError> {
        let sql = format!("SELECT {COLS} FROM skills WHERE id = ?1");
        let mut stmt = self.db.prepare(&sql)?;
        let mut rows = stmt.query_map(params![id], from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    fn require(&self, id: &str) -> Result<Skill, StoreError> {
        self.get(id)?
            .ok_or_else(|| StoreError::not_found(format!("技能不存在：{id}")))
    }

    pub fn create(&self, s: &Skill) -> Result<Skill, StoreError> {
        let mut row = s.clone();
        if let Some(parent) = &s.parent_id {
            if self.get(parent)?.is_none() {
                return Err(StoreError::not_found(format!("父技能不存在：{parent}")));
            }
        }
        if row.id.is_empty() {
            row.id = generate_id();
        }
        if row.created_at.is_empty() {
            row.created_at = today_iso();
        }
        self.db.execute(
            &format!(
                "INSERT INTO skills ({COLS}) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)"
            ),
            params![
                row.id,
                row.name,
                row.description,
                row.parent_id,
                row.category.as_str(),
                row.difficulty.as_str(),
                row.curve_type.map(|c| c.as_str()),
                row.c,
                row.v,
                row.last_settled_date,
                row.created_at,
                row.archived_at,
                row.color,
                row.icon,
                row.sort,
                row.is_branch,
            ],
        )?;
        Ok(row)
    }

    /// 全列覆盖（档案字段合并由调用方完成）；parent 变化时做存在性与环检测。
    pub fn update(&self, s: &Skill) -> Result<Skill, StoreError> {
        let existing = self.require(&s.id)?;
        if s.parent_id != existing.parent_id {
            if let Some(new_parent) = &s.parent_id {
                if new_parent == &s.id {
                    return Err(StoreError::constraint(
                        StoreErrorCode::Cycle,
                        format!("技能 {} 不能把自己设为父级", s.id),
                    ));
                }
                if self.get(new_parent)?.is_none() {
                    return Err(StoreError::not_found(format!("父技能不存在：{new_parent}")));
                }
                let desc = subtree_sql(&s.id, self.db)?;
                if desc.contains(new_parent) {
                    return Err(StoreError::constraint(
                        StoreErrorCode::Cycle,
                        format!("父级移动被拒：{new_parent} 位于 {} 的子树内，移动将形成环", s.id),
                    ));
                }
            }
        }
        self.db.execute(
            "UPDATE skills SET name=?2, description=?3, parent_id=?4, category=?5, difficulty=?6,
             curve_type=?7, c=?8, v=?9, last_settled_date=?10, created_at=?11, archived_at=?12,
             color=?13, icon=?14, sort=?15, is_branch=?16 WHERE id=?1",
            params![
                s.id,
                s.name,
                s.description,
                s.parent_id,
                s.category.as_str(),
                s.difficulty.as_str(),
                s.curve_type.map(|c| c.as_str()),
                s.c,
                s.v,
                s.last_settled_date,
                s.created_at,
                s.archived_at,
                s.color,
                s.icon,
                s.sort,
                s.is_branch,
            ],
        )?;
        Ok(s.clone())
    }

    pub fn archive(&self, id: &str, date: Option<&str>) -> Result<Skill, StoreError> {
        let existing = self.require(id)?;
        if existing.archived_at.is_some() {
            return Ok(existing);
        }
        let archived_at = date.unwrap_or(&today_iso()).to_string();
        self.db
            .execute("UPDATE skills SET archived_at = ?2 WHERE id = ?1", params![id, archived_at])?;
        let mut s = existing;
        s.archived_at = Some(archived_at);
        Ok(s)
    }

    pub fn restore(&self, id: &str) -> Result<Skill, StoreError> {
        let existing = self.require(id)?;
        if existing.archived_at.is_none() {
            return Ok(existing);
        }
        self.db
            .execute("UPDATE skills SET archived_at = NULL WHERE id = ?1", params![id])?;
        let mut s = existing;
        s.archived_at = None;
        Ok(s)
    }

    /// 结算账户写入：只写 c / v / last_settled_date 三列。
    pub fn settle_accounts(
        &self,
        id: &str,
        c: f64,
        v: f64,
        last_settled_date: &str,
    ) -> Result<Skill, StoreError> {
        let existing = self.require(id)?;
        self.db.execute(
            "UPDATE skills SET c = ?2, v = ?3, last_settled_date = ?4 WHERE id = ?1",
            params![id, c, v, last_settled_date],
        )?;
        let mut s = existing;
        s.c = c;
        s.v = v;
        s.last_settled_date = Some(last_settled_date.to_string());
        Ok(s)
    }

    /// 物理删除：有子技能或打卡历史时拒绝。
    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.require(id)?;
        let desc = subtree_sql(id, self.db)?;
        if desc.len() > 1 {
            return Err(StoreError::constraint(
                StoreErrorCode::HasChildren,
                format!("技能 {id} 仍有 {} 个子技能，不能删除（可先归档）", desc.len() - 1),
            ));
        }
        let history: i64 = self.db.query_row(
            "SELECT COUNT(*) AS n FROM daily_record_skills WHERE skill_id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        if history > 0 {
            return Err(StoreError::constraint(
                StoreErrorCode::HasHistory,
                format!("技能 {id} 已有 {history} 条打卡历史，不能物理删除，请先归档"),
            ));
        }
        self.db.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn children_of(&self, id: &str) -> Result<Vec<Skill>, StoreError> {
        let sql = format!("SELECT {COLS} FROM skills WHERE parent_id = ?1 ORDER BY sort ASC, name ASC");
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params![id], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 自身 + 全部后代 id。
    pub fn subtree_ids(&self, id: &str) -> Result<Vec<String>, StoreError> {
        subtree_sql(id, self.db)
    }

    /// 祖先链（根在前、直接父在末）。
    pub fn ancestors_of(&self, id: &str) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.db.prepare(
            "WITH RECURSIVE anc(id, parent_id, depth) AS (
               SELECT id, parent_id, 0 FROM skills WHERE id = ?1
               UNION ALL
               SELECT s.id, s.parent_id, anc.depth + 1
                 FROM skills s JOIN anc ON s.id = anc.parent_id
             ) SELECT id FROM anc WHERE id != ?1 ORDER BY depth DESC",
        )?;
        let rows = stmt.query_map(params![id], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
