//! Budget rules queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetRow {
    pub id: i64,
    pub provider: String,
    pub model: Option<String>,
    pub period: String,
    pub limit_usd: f64,
    pub enabled: bool,
}

pub fn list(conn: &SharedConnection) -> rusqlite::Result<Vec<BudgetRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, provider, model, period, limit_usd, enabled FROM budget_rules ORDER BY id"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BudgetRow {
            id: row.get(0)?,
            provider: row.get(1)?,
            model: row.get(2)?,
            period: row.get(3)?,
            limit_usd: row.get(4)?,
            enabled: row.get::<_, i32>(5)? != 0,
        })
    })?;
    rows.collect()
}

pub fn insert(conn: &SharedConnection, b: &BudgetRow) -> rusqlite::Result<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO budget_rules(provider, model, period, limit_usd, enabled)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![b.provider, b.model, b.period, b.limit_usd, b.enabled as i32],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn used_today(conn: &SharedConnection, provider: &str, model: Option<&str>) -> rusqlite::Result<f64> {
    let c = conn.lock().unwrap();
    let sql = if model.is_some() {
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage
         WHERE date(timestamp) = date('now') AND provider = ?1 AND model = ?2"
    } else {
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage
         WHERE date(timestamp) = date('now') AND provider = ?1"
    };
    match model {
        Some(m) => c.query_row(sql, params![provider, m], |r| r.get::<_, f64>(0)),
        None => c.query_row(sql, params![provider], |r| r.get::<_, f64>(0)),
    }
}
