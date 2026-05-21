//! OpenEva 平台扫描器
//!
//! 判定证据：
//! - ~/.openeva/settings.json
//! - ~/.openeva/mcp.json（顶层 key 为 `servers`，与 Claude Code 的 `mcpServers` 不同）

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;
use std::fs;

pub struct OpenEvaPlatform;

impl PlatformScanner for OpenEvaPlatform {
    fn id(&self) -> &'static str { "openeva" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = home_path(".openeva") else { return Ok(vec![]); };

        let candidates = vec![
            ConfigCandidate::new(home.join("settings.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("mcp.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("auth.json"), ConfigFormat::JsonRelaxed),
        ];

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
            platform: "openeva".into(),
            agent_name: "OpenEva".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("openeva".into()),
            last_seen: now_unix(),
            mcp_servers,
            config_hash: None,
            status: status_label(&res.status).into(),
            discovery_signals: vec![],
        }])
    }
}

fn parse_mcp_servers(home: &std::path::Path) -> Option<Vec<String>> {
    let p = home.join("mcp.json");
    let content = fs::read_to_string(&p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let servers = v
        .get("servers")
        .or_else(|| v.get("mcpServers"))
        .and_then(|s| s.as_object())?;
    Some(servers.keys().cloned().collect())
}

fn now_unix() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", now)
}
