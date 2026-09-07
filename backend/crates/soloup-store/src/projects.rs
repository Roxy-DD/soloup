//! Project Repository（对应 TS `projects.ts`）。状态迁移由 service 层判定。

use rusqlite::{params, Connection, Row};

use soloup_core::schema::{DbEnum, Project, ProjectStatus};

use crate::errors::StoreError;
use crate::generate_id;

pub struct ProjectRepo<'a> {
    pub db: &'a Connection,
}

fn from_row(row: &Row) -> rusqlite::Result<Project> {
    let status: String = row.get(5)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        start_date: row.get(3)?,
        end_date: row.get(4)?,
        status: ProjectStatus::parse(&status).unwrap_or(ProjectStatus::Planned),
        color: row.get(6)?,
    })
}

impl ProjectRepo<'_> {
    pub fn list(&self) -> Result<Vec<Project>, StoreError> {
        let mut stmt = self
            .db
            .prepare("SELECT id, name, description, start_date, end_date, status, color FROM projects ORDER BY start_date ASC, name ASC")?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, id: &str) -> Result<Option<Project>, StoreError> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, description, start_date, end_date, status, color FROM projects WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], from_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn create(&self, p: &Project) -> Result<Project, StoreError> {
        let mut row = p.clone();
        if row.id.is_empty() {
            row.id = generate_id();
        }
        self.db.execute(
            "INSERT INTO projects (id, name, description, start_date, end_date, status, color)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                row.id,
                row.name,
                row.description,
                row.start_date,
                row.end_date,
                row.status.as_str(),
                row.color,
            ],
        )?;
        Ok(row)
    }

    pub fn update(&self, p: &Project) -> Result<Project, StoreError> {
        let n = self.db.execute(
            "UPDATE projects SET name=?2, description=?3, start_date=?4, end_date=?5, status=?6, color=?7 WHERE id=?1",
            params![
                p.id,
                p.name,
                p.description,
                p.start_date,
                p.end_date,
                p.status.as_str(),
                p.color,
            ],
        )?;
        if n == 0 {
            return Err(StoreError::not_found(format!("项目不存在：{}", p.id)));
        }
        Ok(p.clone())
    }

    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.db.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }
}
