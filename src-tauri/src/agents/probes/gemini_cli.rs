//! Gemini CLI 反向导入
//!
//! Gemini CLI 没有多 provider 配置，只有 ~/.gemini/.env 单个 GEMINI_API_KEY。
//! 不显式存 base_url（CLI 走默认 generativelanguage.googleapis.com）。

use crate::agents::config_probe::home_dir;
use crate::agents::probes::ChannelCandidate;

pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(home) = home_dir() else { return vec![]; };

    // 1. 读 ~/.gemini/.env 找 GEMINI_API_KEY / GOOGLE_API_KEY
    let env_path = home.join(".gemini/.env");
    let api_key: Option<String> = std::fs::read_to_string(&env_path)
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with('#') {
                    continue;
                }
                if let Some(eq) = t.find('=') {
                    let key = t[..eq].trim();
                    if key == "GEMINI_API_KEY" || key == "GOOGLE_API_KEY" {
                        let raw = t[eq + 1..].trim();
                        let val = raw
                            .trim_start_matches('"')
                            .trim_end_matches('"')
                            .trim_start_matches('\'')
                            .trim_end_matches('\'');
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
            }
            None
        });

    // 2. 读 settings.json 的 security.auth.selectedType 判断登录方式
    // cc-switch 用 GeminiAuthType 枚举：api_key | oauth | login
    let selected_auth: Option<String> = std::fs::read_to_string(home.join(".gemini/settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| {
            v.pointer("/security/auth/selectedType")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
        });
    let is_oauth_login = selected_auth
        .as_ref()
        .map(|t| t.contains("oauth") || t.contains("login") || t.contains("google"))
        .unwrap_or(false);

    // 完全无 key 且非 OAuth 模式 → 不返回候选（用户根本没配置过）
    if api_key.is_none() && !is_oauth_login {
        return vec![];
    }

    let mut warnings = Vec::new();
    if api_key.is_none() && is_oauth_login {
        warnings.push(
            "Gemini Google 账号登录模式：本机无原生 API key，需用户手动补一个直连 key 才能用 ClawHeart 代理"
                .into(),
        );
    }

    vec![ChannelCandidate {
        id: "gemini:google".to_string(),
        name: "Google Gemini".to_string(),
        source_agent_id: agent_id.to_string(),
        source_platform: "gemini".to_string(),
        base_url: "https://generativelanguage.googleapis.com".to_string(),
        api_key,
        protocol: "gemini".to_string(),
        default_model: None,
        provider_kind: "custom".to_string(),
        already_exists: false,
        warnings,
    }]
}
