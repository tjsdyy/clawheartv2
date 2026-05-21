//! Security rule overrides queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RuleOverride {
    pub rule_kind: String,
    pub rule_id: String,
    pub enabled: bool,
    pub action: Option<String>,
    pub updated_at: String,
}

pub fn list_all(conn: &SharedConnection) -> rusqlite::Result<Vec<RuleOverride>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT rule_kind, rule_id, enabled, action, updated_at
         FROM security_rule_overrides",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(RuleOverride {
            rule_kind: row.get(0)?,
            rule_id: row.get(1)?,
            enabled: row.get::<_, i32>(2)? != 0,
            action: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn list_by_kind(
    conn: &SharedConnection,
    kind: &str,
) -> rusqlite::Result<Vec<RuleOverride>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT rule_kind, rule_id, enabled, action, updated_at
         FROM security_rule_overrides WHERE rule_kind = ?1",
    )?;
    let rows = stmt.query_map(params![kind], |row| {
        Ok(RuleOverride {
            rule_kind: row.get(0)?,
            rule_id: row.get(1)?,
            enabled: row.get::<_, i32>(2)? != 0,
            action: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn upsert(
    conn: &SharedConnection,
    kind: &str,
    id: &str,
    enabled: bool,
    action: Option<&str>,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO security_rule_overrides(rule_kind, rule_id, enabled, action, updated_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))
         ON CONFLICT(rule_kind, rule_id) DO UPDATE SET
            enabled = excluded.enabled,
            action = excluded.action,
            updated_at = datetime('now')",
        params![kind, id, enabled as i32, action],
    )?;
    Ok(())
}

pub fn reset(conn: &SharedConnection, kind: &str, id: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "DELETE FROM security_rule_overrides WHERE rule_kind = ?1 AND rule_id = ?2",
        params![kind, id],
    )?;
    Ok(())
}

pub fn reset_kind(conn: &SharedConnection, kind: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "DELETE FROM security_rule_overrides WHERE rule_kind = ?1",
        params![kind],
    )?;
    Ok(())
}
