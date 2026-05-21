//! OpenAI Codex 平台扫描器
//!
//! 判定证据：
//! - ~/.codex/auth.json（登录凭据，最强证据）
//! - ~/.codex/config.toml（配置项，弱证据）
//!
//! 关键字段：auth.json 中需含 `OPENAI_API_KEY` 或 `tokens.access_token`
//! 或同等结构；为简化，只要可解析为 JSON 即视为有效。

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;

pub struct CodexPlatform;

impl PlatformScanner for CodexPlatform {
    fn id(&self) -> &'static str { "codex" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = home_path(".codex") else { return Ok(vec![]); };

        let candidates = vec![
            ConfigCandidate::new(home.join("auth.json"), ConfigFormat::Json),
            ConfigCandidate::new(home.join("config.toml"), ConfigFormat::Toml),
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
            platform: "codex".into(),
            agent_name: "Codex CLI".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("codex".into()),
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
