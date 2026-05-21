//! Codeium Windsurf 扫描器（跨平台 fallback 路径）

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;

pub struct WindsurfPlatform;

impl PlatformScanner for WindsurfPlatform {
    fn id(&self) -> &'static str { "windsurf" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let roots = [
            home_path("Library/Application Support/Windsurf"),
            home_path(".config/Windsurf"),
            home_path("AppData/Roaming/Windsurf"),
        ];

        let mut candidates: Vec<ConfigCandidate> = Vec::new();
        for root in roots.into_iter().flatten() {
            candidates.push(ConfigCandidate::new(
                root.join("User/settings.json"),
                ConfigFormat::JsonRelaxed,
            ));
            candidates.push(ConfigCandidate::new(
                root.join("User/globalStorage/storage.json"),
                ConfigFormat::JsonRelaxed,
            ));
            candidates.push(ConfigCandidate::new(
                root.join("User/keybindings.json"),
                ConfigFormat::JsonRelaxed,
            ));
        }

        let res = detect_config(&candidates);
        if res.status == DetectionStatus::NotFound {
            return Ok(vec![]);
        }

        let path = res
            .matched_path
            .as_ref()
            .and_then(|p| p.parent().and_then(|p| p.parent()))
            .map(|p| p.to_path_buf())
            .or_else(|| res.matched_path.clone())
            .unwrap_or_default();

        Ok(vec![DiscoveredAgent {
            platform: "windsurf".into(),
            agent_name: "Windsurf".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("Windsurf".into()),
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
