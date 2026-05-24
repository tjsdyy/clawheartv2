//! `clawheart providers` — Provider Profile 管理
//!
//! 注：本 alpha 版仅暴露 `list`；`add` / `import` / `overwrite` 后续接入。
use clap::Subcommand;
use serde::Serialize;
use std::path::PathBuf;

use super::output::{CliResult, Output};

#[derive(Subcommand)]
pub enum ProvidersCmd {
    /// 列出当前所有 Provider Profile
    List,
    /// 交互式新增（后续接入）
    Add,
    /// 从 Agent 配置导入候选（后续接入）
    Import,
    /// 一键覆盖到所有 Agent（后续接入）
    Overwrite { profile_id: String },
}

#[derive(Serialize)]
struct ProviderDto {
    id: String,
    name: String,
    provider_kind: String,
    protocol: String,
    base_url: String,
    virtual_key: String,
    is_default: bool,
    enabled: bool,
}

pub fn execute(cmd: ProvidersCmd, json: bool, db: Option<PathBuf>) -> CliResult {
    match cmd {
        ProvidersCmd::List => list(json, db),
        ProvidersCmd::Add | ProvidersCmd::Import | ProvidersCmd::Overwrite { .. } => {
            Output::error("尚未实现 · 当前 alpha 阶段请在桌面 GUI 操作").emit(json);
            Ok(())
        }
    }
}

#[cfg(feature = "storage")]
fn list(json: bool, db_override: Option<PathBuf>) -> CliResult {
    use crate::storage::conn;
    use crate::storage::queries::providers as q;

    let db_path = db_override.unwrap_or_else(conn::default_path);
    let db = conn::open(&db_path).map_err(|e| format!("open DB: {}", e))?;
    let rows = q::list(&db).map_err(|e| format!("query: {}", e))?;

    let dtos: Vec<ProviderDto> = rows
        .iter()
        .map(|r| ProviderDto {
            id: r.id.clone(),
            name: r.name.clone(),
            provider_kind: r.provider_kind.clone(),
            protocol: r.protocol.clone(),
            base_url: r.base_url.clone(),
            virtual_key: r.virtual_key.clone(),
            is_default: r.is_default,
            enabled: r.enabled,
        })
        .collect();

    let text = if dtos.is_empty() {
        "无 Provider Profile（在桌面端「中转配置」中新建）".into()
    } else {
        let mut s = format!("✓ {} 个 Profile\n\n", dtos.len());
        for p in &dtos {
            s.push_str(&format!(
                "  [{}] {}{}  {} · {} → {}\n",
                p.id,
                p.name,
                if p.is_default { " ★default" } else { "" },
                p.provider_kind,
                p.protocol,
                p.base_url,
            ));
            s.push_str(&format!("      vkey: {}\n", p.virtual_key));
        }
        s
    };

    Output::ok_with_text(dtos, text).emit(json);
    Ok(())
}

#[cfg(not(feature = "storage"))]
fn list(json: bool, _db: Option<PathBuf>) -> CliResult {
    Output::error("CLI 编译时未启用 storage feature").emit(json);
    Ok(())
}
