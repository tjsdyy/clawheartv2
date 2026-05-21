//! OpenClaw 平台扫描器
//!
//! 判定证据：
//! - ~/.openclaw/openclaw.json
//! - ~/.openclaw/config.json
//! - ~/.openclaw/settings.json

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;

pub struct OpenClawPlatform;

impl PlatformScanner for OpenClawPlatform {
    fn id(&self) -> &'static str { "openclaw" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = home_path(".openclaw") else { return Ok(vec![]); };

        let candidates = vec![
            ConfigCandidate::new(home.join("openclaw.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("config.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("settings.json"), ConfigFormat::JsonRelaxed),
        ];

        let res = detect_config(&candidates);
        if res.status == DetectionStatus::NotFound {
            return Ok(vec![]);
        }

        let path = res
            .matched_path
            .clone()
            .unwrap_or_else(|| home.clone());

        Ok(vec![DiscoveredAgent {
            platform: "openclaw".into(),
            agent_name: "OpenClaw".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("openclaw".into()),
            last_seen: now_unix(),
            mcp_servers: vec![],
            config_hash: None,
            status: status_label(&res.status).into(),
            discovery_signals: vec![],
        }])
    }
}

fn now_unix() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", now)
}
