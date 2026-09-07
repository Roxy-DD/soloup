//! Attribute Repository（对应 TS `attributes.ts`）。

use rusqlite::{params, Connection, Row};

use soloup_core::schema::{Attribute, AttributeCategory, DbEnum};

use crate::errors::{StoreError, StoreErrorCode};
use crate::generate_id;

pub const COLS: &str =
    "id, name, description, base_value, max_value, alpha, w0, category, color, icon, sort";

pub struct AttributeRepo<'a> {
    pub db: &'a Connection,
}

fn opt_enum<E: DbEnum>(s: Option<String>) -> Result<Option<E>, StoreError> {
    match s {
        None => Ok(None),
        Some(v) => E::parse(&v).map(Some).ok_or_else(|| {
            StoreError::new(StoreErrorCode::Validation, format!("非法枚举值：{v}"))
        }),
    }
}

fn from_row(row: &Row) -> rusqlite::Result<Attribute> {
    Ok(Attribute {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        base_value: row.get(3)?,
        max_value: row.get(4)?,
        alpha: row.get(5)?,
        w0: row.get(6)?,
        category: opt_enum::<AttributeCategory>(row.get(7)?).unwrap_or(None),
        color: row.get(8)?,
        icon: row.get(9)?,
        sort: row.get(10)?,
    })
}

impl AttributeRepo<'_> {
    pub fn list(&self) -> Result<Vec<Attribute>, StoreError> {
        let sql = format!("SELECT {COLS} FROM attributes ORDER BY sort ASC, name ASC");
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, id: &str) -> Result<Option<Attribute>, StoreError> {
        let sql = format!("SELECT {COLS} FROM attributes WHERE id = ?");
        let mut stmt = self.db.prepare(&sql)?;
        let mut rows = stmt.query_map(params![id], from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// 插入（id 为空则生成 UUID）。返回落库行。
    pub fn create(&self, a: &Attribute) -> Result<Attribute, StoreError> {
        let row = Attribute {
            id: if a.id.is_empty() { generate_id() } else { a.id.clone() },
            ..a.clone()
        };
        self.db.execute(
            &format!(
                "INSERT INTO attributes ({COLS}) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"
            ),
            params![
                row.id,
                row.name,
                row.description,
                row.base_value,
                row.max_value,
                row.alpha,
                row.w0,
                row.category.map(|c| c.as_str()),
                row.color,
                row.icon,
                row.sort,
            ],
        )?;
        Ok(row)
    }

    /// 全列覆盖（调用方负责先 get 再合并 patch）。
    pub fn update(&self, a: &Attribute) -> Result<Attribute, StoreError> {
        let n = self.db.execute(
            "UPDATE attributes SET name=?2, description=?3, base_value=?4, max_value=?5,
             alpha=?6, w0=?7, category=?8, color=?9, icon=?10, sort=?11 WHERE id=?1",
            params![
                a.id,
                a.name,
                a.description,
                a.base_value,
                a.max_value,
                a.alpha,
                a.w0,
                a.category.map(|c| c.as_str()),
                a.color,
                a.icon,
                a.sort,
            ],
        )?;
        if n == 0 {
            return Err(StoreError::not_found(format!("属性不存在：{}", a.id)));
        }
        Ok(a.clone())
    }

    /// 物理删除；仍被技能关联时拒绝（ERR_LINKED）。
    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        let linked: i64 = self
            .db
            .query_row(
                "SELECT COUNT(*) AS n FROM skill_attributes WHERE attribute_id = ?",
                params![id],
                |r| r.get(0),
            )?;
        if linked > 0 {
            return Err(StoreError::constraint(
                StoreErrorCode::Linked,
                format!("属性 {id} 仍被 {linked} 个技能关联，请先解除关联再删除"),
            ));
        }
        self.db.execute("DELETE FROM attributes WHERE id = ?", params![id])?;
        Ok(())
    }
}
