//! Token usage / request log queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::RequestLog;
use rusqlite::params;

pub fn insert_request_log(conn: &SharedConnection, log: &RequestLog) -> rusqlite::Result<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO request_logs(timestamp, agent_id, format, provider, model, endpoint,
                                 method, status_code, blocked, bytes_in, bytes_out, latency_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            log.timestamp, log.agent_id, log.format, log.provider, log.model,
            log.endpoint, log.method, log.status_code, log.blocked as i32,
            log.bytes_in, log.bytes_out, log.latency_ms,
        ],
    )?;
    Ok(c.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub fn insert_token_usage(
    conn: &SharedConnection,
    request_log_id: Option<i64>,
    agent_id: Option<&str>,
    provider: &str,
    model: &str,
    input: u32,
    output: u32,
    cache_read: u32,
    cache_creation: u32,
    cost_usd: f64,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO token_usage(timestamp, request_log_id, agent_id, provider, model,
                                input_tokens, output_tokens, cache_read, cache_creation, cost_usd)
         VALUES (datetime('now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![request_log_id, agent_id, provider, model, input, output, cache_read, cache_creation, cost_usd],
    )?;
    Ok(())
}

/// 按日聚合 token 用量（最近 N 天）；返回 (date, input, output, cost)
pub fn daily_usage_summary(
    conn: &SharedConnection,
    days: u32,
) -> rusqlite::Result<Vec<(String, u32, u32, f64)>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT date(timestamp) AS d,
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cost_usd), 0)
         FROM token_usage
         WHERE timestamp >= datetime('now', ?1)
         GROUP BY d
         ORDER BY d DESC",
    )?;
    let offset = format!("-{} days", days);
    let rows = stmt.query_map(params![offset], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;
    rows.collect()
}

/// 今日总览：(input, output, cache_read, cost, request_count, blocked_count)
pub fn today_summary(conn: &SharedConnection) -> rusqlite::Result<(u32, u32, u32, f64, u32, u32)> {
    let c = conn.lock().unwrap();
    let (input, output, cache, cost): (u32, u32, u32, f64) = c.query_row(
        "SELECT COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read), 0),
                COALESCE(SUM(cost_usd), 0)
         FROM token_usage WHERE date(timestamp) = date('now')",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    let (req, blk): (u32, u32) = c.query_row(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN blocked THEN 1 ELSE 0 END), 0)
         FROM request_logs WHERE date(timestamp) = date('now')",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok((input, output, cache, cost, req, blk))
}

/// 按 provider 汇总（最近 N 天）
pub fn usage_by_provider(
    conn: &SharedConnection,
    days: u32,
) -> rusqlite::Result<Vec<(String, u32, u32, f64, u32)>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT provider,
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cost_usd), 0),
                COUNT(*) AS req_count
         FROM token_usage
         WHERE timestamp >= datetime('now', ?1)
         GROUP BY provider
         ORDER BY SUM(cost_usd) DESC",
    )?;
    let offset = format!("-{} days", days);
    let rows = stmt.query_map(params![offset], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, u32>(1)?,
            r.get::<_, u32>(2)?,
            r.get::<_, f64>(3)?,
            r.get::<_, u32>(4)?,
        ))
    })?;
    rows.collect()
}

/// 按 model 汇总（最近 N 天）
pub fn usage_by_model(
    conn: &SharedConnection,
    days: u32,
) -> rusqlite::Result<Vec<(String, String, u32, u32, f64, u32)>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT provider, model,
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cost_usd), 0),
                COUNT(*) AS req_count
         FROM token_usage
         WHERE timestamp >= datetime('now', ?1)
         GROUP BY provider, model
         ORDER BY SUM(cost_usd) DESC",
    )?;
    let offset = format!("-{} days", days);
    let rows = stmt.query_map(params![offset], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, u32>(2)?,
            r.get::<_, u32>(3)?,
            r.get::<_, f64>(4)?,
            r.get::<_, u32>(5)?,
        ))
    })?;
    rows.collect()
}

pub fn list_request_logs(conn: &SharedConnection, limit: u32) -> rusqlite::Result<Vec<RequestLog>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, timestamp, agent_id, format, provider, model, endpoint, method,
                status_code, blocked, bytes_in, bytes_out, latency_ms
         FROM request_logs ORDER BY timestamp DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(RequestLog {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            agent_id: row.get(2)?,
            format: row.get(3)?,
            provider: row.get(4)?,
            model: row.get(5)?,
            endpoint: row.get(6)?,
            method: row.get(7)?,
            status_code: row.get(8)?,
            blocked: row.get::<_, i32>(9)? != 0,
            bytes_in: row.get(10)?,
            bytes_out: row.get(11)?,
            latency_ms: row.get(12)?,
        })
    })?;
    rows.collect()
}
