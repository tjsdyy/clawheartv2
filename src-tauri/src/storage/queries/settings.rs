//! Settings queries — rusqlite

#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::Setting;
use rusqlite::params;

pub fn get(conn: &SharedConnection, key: &str) -> rusqlite::Result<Option<String>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn set(conn: &SharedConnection, key: &str, value: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    set_with_conn(&c, key, value)
}

/// 在已持有锁的 Connection 上执行 set —— 避免 seed.rs 中嵌套锁死锁
pub fn set_with_conn(c: &rusqlite::Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    c.execute(
        "INSERT INTO settings(key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
        params![key, value],
    )?;
    Ok(())
}

pub fn delete(conn: &SharedConnection, key: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}

pub fn list_all(conn: &SharedConnection) -> rusqlite::Result<Vec<Setting>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT key, value, updated_at FROM settings ORDER BY key")?;
    let rows = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    rows.collect()
}
