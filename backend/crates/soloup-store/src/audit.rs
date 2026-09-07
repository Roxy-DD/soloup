//! AuditLog Repository（对应 TS `audit.ts`）：全写入审计（actor = user|ai|system）。

use chrono::Utc;
use rusqlite::{params, Connection, Row};
use serde_json::Value;

use soloup_core::schema::{AuditActor, AuditLog, DbEnum};

use crate::errors::StoreError;

pub struct AuditRepo<'a> {
    pub db: &'a Connection,
}

fn from_row(row: &Row) -> rusqlite::Result<AuditLog> {
    let actor: String = row.get(2)?;
    let params_text: Option<String> = row.get(4)?;
    Ok(AuditLog {
        id: row.get(0)?,
        at: row.get(1)?,
        actor: AuditActor::parse(&actor).unwrap_or(AuditActor::User),
        tool: row.get(3)?,
        params_json: params_text
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
    })
}

#[derive(Default)]
pub struct AuditQuery {
    pub limit: i64,
    pub offset: i64,
    pub actor: Option<AuditActor>,
    pub tool: Option<String>,
}

impl AuditRepo<'_> {
    /// 写入一条审计并返回落库行。
    pub fn append(
        &self,
        actor: AuditActor,
        tool: Option<&str>,
        params: Option<&Value>,
    ) -> Result<AuditLog, StoreError> {
        let params_json = params.map(|v| v.to_string());
        let at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        self.db.execute(
            "INSERT INTO audit_log (at, actor, tool, params_json) VALUES (?1, ?2, ?3, ?4)",
            params![at, actor.as_str(), tool, params_json],
        )?;
        let id = self.db.last_insert_rowid();
        let mut stmt = self.db.prepare("SELECT id, at, actor, tool, params_json FROM audit_log WHERE id = ?")?;
        let row = stmt.query_row(params![id], from_row)?;
        Ok(row)
    }

    /// 最近写入在前（按 id 倒序）。
    pub fn list(&self, q: &AuditQuery) -> Result<Vec<AuditLog>, StoreError> {
        let limit = if q.limit > 0 { q.limit } else { 100 };
        let offset = q.offset.max(0);
        let mut where_clauses: Vec<&str> = vec![];
        let mut args: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
        if let Some(a) = q.actor {
            where_clauses.push("actor = ?");
            args.push(Box::new(a.as_str().to_string()));
        }
        if let Some(t) = &q.tool {
            where_clauses.push("tool = ?");
            args.push(Box::new(t.clone()));
        }
        let sql = format!(
            "SELECT id, at, actor, tool, params_json FROM audit_log {} ORDER BY id DESC LIMIT ? OFFSET ?",
            if where_clauses.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_clauses.join(" AND "))
            }
        );
        let mut stmt = self.db.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(
            args.iter().map(|a| a.as_ref()).chain([
                &limit as &dyn rusqlite::types::ToSql,
                &offset as &dyn rusqlite::types::ToSql,
            ]),
        ))?;
        let mut out = Vec::new();
        while let Some(r) = rows.next()? {
            out.push(from_row(r)?);
        }
        Ok(out)
    }
}
