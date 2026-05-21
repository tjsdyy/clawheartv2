//! Provider profile queries — 第三方 LLM 中转 API 集中管理
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfileRow {
    pub id: String,
    pub name: String,
    pub provider_kind: String,
    pub protocol: String,
    pub base_url: String,
    pub credential_ref: String,
    pub default_model: Option<String>,
    pub headers_json: Option<String>,
    pub virtual_key: String,
    pub is_default: bool,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn row_from(row: &rusqlite::Row) -> rusqlite::Result<ProviderProfileRow> {
    Ok(ProviderProfileRow {
        id: row.get(0)?,
        name: row.get(1)?,
        provider_kind: row.get(2)?,
        protocol: row.get(3)?,
        base_url: row.get(4)?,
        credential_ref: row.get(5)?,
        default_model: row.get(6)?,
        headers_json: row.get(7)?,
        virtual_key: row.get(8)?,
        is_default: row.get::<_, i32>(9)? != 0,
        enabled: row.get::<_, i32>(10)? != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SELECT_COLS: &str =
    "id, name, provider_kind, protocol, base_url, credential_ref, default_model, \
     headers_json, virtual_key, is_default, enabled, created_at, updated_at";

pub fn list(conn: &SharedConnection) -> rusqlite::Result<Vec<ProviderProfileRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(&format!(
        "SELECT {} FROM provider_profiles ORDER BY is_default DESC, created_at",
        SELECT_COLS
    ))?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect()
}

pub fn get(
    conn: &SharedConnection,
    id: &str,
) -> rusqlite::Result<Option<ProviderProfileRow>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        &format!("SELECT {} FROM provider_profiles WHERE id = ?1", SELECT_COLS),
        params![id],
        row_from,
    ) {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_by_virtual_key(
    conn: &SharedConnection,
    virtual_key: &str,
) -> rusqlite::Result<Option<ProviderProfileRow>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        &format!(
            "SELECT {} FROM provider_profiles WHERE virtual_key = ?1",
            SELECT_COLS
        ),
        params![virtual_key],
        row_from,
    ) {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn count_default(conn: &SharedConnection) -> rusqlite::Result<u32> {
    let c = conn.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM provider_profiles WHERE is_default = 1",
        [],
        |row| row.get::<_, u32>(0),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &SharedConnection,
    id: &str,
    name: &str,
    provider_kind: &str,
    protocol: &str,
    base_url: &str,
    credential_ref: &str,
    default_model: Option<&str>,
    headers_json: Option<&str>,
    virtual_key: &str,
    is_default: bool,
    enabled: bool,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO provider_profiles
            (id, name, provider_kind, protocol, base_url, credential_ref,
             default_model, headers_json, virtual_key, is_default, enabled,
             created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                 datetime('now'), datetime('now'))",
        params![
            id,
            name,
            provider_kind,
            protocol,
            base_url,
            credential_ref,
            default_model,
            headers_json,
            virtual_key,
            is_default as i32,
            enabled as i32,
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &SharedConnection,
    id: &str,
    name: &str,
    provider_kind: &str,
    protocol: &str,
    base_url: &str,
    default_model: Option<&str>,
    headers_json: Option<&str>,
    enabled: bool,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE provider_profiles SET
            name = ?2, provider_kind = ?3, protocol = ?4, base_url = ?5,
            default_model = ?6, headers_json = ?7, enabled = ?8,
            updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            name,
            provider_kind,
            protocol,
            base_url,
            default_model,
            headers_json,
            enabled as i32,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &SharedConnection, id: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "DELETE FROM provider_profiles WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn clear_default(conn: &SharedConnection) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE provider_profiles SET is_default = 0, updated_at = datetime('now')",
        [],
    )?;
    Ok(())
}

pub fn mark_default(conn: &SharedConnection, id: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE provider_profiles SET is_default = 1, updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
