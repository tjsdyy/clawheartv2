//! Claude Code 平台扫描器
//!
//! 判定证据（任一可解析即 Active）：
//! - ~/.claude/settings.json（新版主配置）
//! - ~/.claude/claude.json（旧版兼容）
//! - ~/.claude.json（全局 MCP，作为 fallback 信号）
//!
//! 解析失败 → ConfigBroken；都不存在 → 不返回此 Agent。

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;
use std::fs;

pub struct ClaudePlatform;

impl PlatformScanner for ClaudePlatform {
    fn id(&self) -> &'static str { "claude" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = home_path(".claude") else { return Ok(vec![]); };
        let global_mcp = home_path(".claude.json");

        let mut candidates = vec![
            ConfigCandidate::new(home.join("settings.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("claude.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join(".mcp.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("mcp.json"), ConfigFormat::JsonRelaxed),
        ];
        if let Some(g) = global_mcp {
            candidates.push(ConfigCandidate::new(g, ConfigFormat::JsonRelaxed));
        }

        let res = detect_config(&candidates);
        if res.status == DetectionStatus::NotFound {
            return Ok(vec![]);
        }

        let mcp_servers = parse_mcp_servers(&home).unwrap_or_default();
        let path = res
            .matched_path
            .clone()
            .unwrap_or_else(|| home.clone());

        Ok(vec![DiscoveredAgent {
            platform: "claude".into(),
            agent_name: "Claude Code".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("claude".into()),
            last_seen: now_unix(),
            mcp_servers,
            config_hash: None,
            status: status_label(&res.status).into(),
            discovery_signals: vec![],
        }])
    }
}

fn parse_mcp_servers(claude_dir: &std::path::Path) -> Option<Vec<String>> {
    for candidate in &[".mcp.json", "mcp.json"] {
        let p = claude_dir.join(candidate);
        if let Ok(content) = fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(servers) = v.get("mcpServers").and_then(|s| s.as_object()) {
                    return Some(servers.keys().cloned().collect());
                }
            }
        }
    }
    None
}

fn now_unix() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", now)
}
