//! SkillAttributeLink Repository（对应 TS `links.ts`）：叶子技能 ↔ 属性。
//! set_for_skill 全量替换须在调用方事务（Store::tx）内执行以保证原子。

use rusqlite::{params, Connection, Row};

use soloup_core::schema::SkillAttributeLink;

use crate::errors::{StoreError, StoreErrorCode};

#[derive(Debug, Clone)]
pub struct SkillLinkInput {
    pub attribute_id: String,
    pub weight: f64,
}

pub struct LinkRepo<'a> {
    pub db: &'a Connection,
}

fn from_row(row: &Row) -> rusqlite::Result<SkillAttributeLink> {
    Ok(SkillAttributeLink {
        skill_id: row.get(0)?,
        attribute_id: row.get(1)?,
        weight: row.get(2)?,
    })
}

impl LinkRepo<'_> {
    pub fn list_for_skill(&self, skill_id: &str) -> Result<Vec<SkillAttributeLink>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT skill_id, attribute_id, weight FROM skill_attributes WHERE skill_id = ?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![skill_id], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_for_attribute(&self, attribute_id: &str) -> Result<Vec<SkillAttributeLink>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT skill_id, attribute_id, weight FROM skill_attributes WHERE attribute_id = ?1 ORDER BY skill_id",
        )?;
        let rows = stmt.query_map(params![attribute_id], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, skill_id: &str, attribute_id: &str) -> Result<Option<SkillAttributeLink>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT skill_id, attribute_id, weight FROM skill_attributes WHERE skill_id = ?1 AND attribute_id = ?2",
        )?;
        let mut rows = stmt.query_map(params![skill_id, attribute_id], from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// 全量替换某技能的所有关联（传空=清空）。须在事务内调用。
    pub fn set_for_skill(
        &self,
        skill_id: &str,
        links: &[SkillLinkInput],
    ) -> Result<Vec<SkillAttributeLink>, StoreError> {
        let exists: Option<i64> = self
            .db
            .query_row("SELECT 1 FROM skills WHERE id = ?1", params![skill_id], |r| r.get(0))
            .optional()?;
        if exists.is_none() {
            return Err(StoreError::not_found(format!("技能不存在：{skill_id}")));
        }
        let kids: i64 = self.db.query_row(
            "SELECT COUNT(*) AS n FROM skills WHERE parent_id = ?1",
            params![skill_id],
            |r| r.get(0),
        )?;
        if kids > 0 {
            return Err(StoreError::constraint(
                StoreErrorCode::Constraint,
                format!("技能 {skill_id} 是非叶子（含 {kids} 个子技能），仅叶子技能可关联属性"),
            ));
        }
        // 校验属性存在 + 权重范围 + 去重
        let mut deduped: Vec<SkillLinkInput> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for link in links {
            if link.weight < 0.0 || link.weight > 1.0 {
                return Err(StoreError::constraint(
                    StoreErrorCode::Validation,
                    format!("关联权重须在 [0,1]：{}", link.weight),
                ));
            }
            let attr: Option<i64> = self
                .db
                .query_row("SELECT 1 FROM attributes WHERE id = ?1", params![link.attribute_id], |r| r.get(0))
                .optional()?;
            if attr.is_none() {
                return Err(StoreError::not_found(format!("属性不存在：{}", link.attribute_id)));
            }
            if seen.insert(link.attribute_id.clone()) {
                deduped.push(link.clone());
            }
        }
        self.db
            .execute("DELETE FROM skill_attributes WHERE skill_id = ?1", params![skill_id])?;
        for l in &deduped {
            self.db.execute(
                "INSERT INTO skill_attributes (skill_id, attribute_id, weight) VALUES (?1, ?2, ?3)",
                params![skill_id, l.attribute_id, l.weight],
            )?;
        }
        Ok(deduped
            .iter()
            .map(|l| SkillAttributeLink {
                skill_id: skill_id.to_string(),
                attribute_id: l.attribute_id.clone(),
                weight: l.weight,
            })
            .collect())
    }

    pub fn remove(&self, skill_id: &str, attribute_id: &str) -> Result<(), StoreError> {
        self.db.execute(
            "DELETE FROM skill_attributes WHERE skill_id = ?1 AND attribute_id = ?2",
            params![skill_id, attribute_id],
        )?;
        Ok(())
    }
}

use rusqlite::OptionalExtension;
