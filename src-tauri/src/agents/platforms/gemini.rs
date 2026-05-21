//! Gemini CLI 扫描器
//!
//! 判定证据：
//! - ~/.gemini/.env（含非空 `GEMINI_API_KEY` 才算 Active）
//! - ~/.gemini/settings.json（备选）

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;

pub struct GeminiPlatform;

impl PlatformScanner for GeminiPlatform {
    fn id(&self) -> &'static str { "gemini" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = home_path(".gemini") else { return Ok(vec![]); };

        let candidates = vec![
            ConfigCandidate::new(home.join(".env"), ConfigFormat::DotEnv)
                .require_key("GEMINI_API_KEY"),
            ConfigCandidate::new(home.join("settings.json"), ConfigFormat::JsonRelaxed),
            ConfigCandidate::new(home.join("config.json"), ConfigFormat::JsonRelaxed),
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
            platform: "gemini".into(),
            agent_name: "Gemini CLI".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("gemini".into()),
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
