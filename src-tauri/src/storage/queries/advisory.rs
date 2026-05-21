//! Security advisories queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::SecurityAdvisory;
use rusqlite::params;

pub fn upsert(conn: &SharedConnection, adv: &SecurityAdvisory) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO security_advisories(id, severity, title, affected, cvss_score, action, published, fetched_at, dismissed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), ?8)
         ON CONFLICT(id) DO UPDATE SET
            severity = excluded.severity,
            title = excluded.title,
            affected = excluded.affected,
            cvss_score = excluded.cvss_score,
            action = excluded.action,
            fetched_at = datetime('now')",
        params![
            adv.id, adv.severity, adv.title, adv.affected, adv.cvss_score,
            adv.action, adv.published, adv.dismissed as i32,
        ],
    )?;
    Ok(())
}

pub fn list(conn: &SharedConnection) -> rusqlite::Result<Vec<SecurityAdvisory>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, severity, title, affected, cvss_score, action, published, fetched_at, dismissed
         FROM security_advisories ORDER BY published DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SecurityAdvisory {
            id: row.get(0)?,
            severity: row.get(1)?,
            title: row.get(2)?,
            affected: row.get(3)?,
            cvss_score: row.get(4)?,
            action: row.get(5)?,
            published: row.get(6)?,
            fetched_at: row.get(7)?,
            dismissed: row.get::<_, i32>(8)? != 0,
        })
    })?;
    rows.collect()
}

pub fn dismiss(conn: &SharedConnection, id: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute("UPDATE security_advisories SET dismissed = 1 WHERE id = ?1", params![id])?;
    Ok(())
}
