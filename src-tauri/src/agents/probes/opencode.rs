//! OpenCode 反向导入
//!
//! 配置文件：~/.config/opencode/opencode.json（XDG style）
//! 多 provider 结构：`provider.<name>.options.{baseURL, apiKey}`
//! 参考：cc-switch live.rs:741-798 / 9router opencode-settings/route.js

use crate::agents::config_probe::home_dir;
use crate::agents::probes::ChannelCandidate;

pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(home) = home_dir() else { return vec![]; };
    let path = home.join(".config/opencode/opencode.json");

    let Ok(content) = std::fs::read_to_string(&path) else { return vec![]; };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { return vec![]; };

    let Some(providers) = v.get("provider").and_then(|p| p.as_object()) else {
        return vec![];
    };

    let mut out = Vec::new();
    for (key, cfg) in providers {
        let options = cfg.get("options").and_then(|o| o.as_object());

        // OpenCode 把 baseURL/apiKey 放在 options 子对象下
        let base_url = options
            .and_then(|o| {
                o.get("baseURL")
                    .or_else(|| o.get("base_url"))
                    .or_else(|| o.get("baseUrl"))
            })
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        if base_url.is_empty() {
            continue;
        }

        let api_key = options
            .and_then(|o| {
                o.get("apiKey")
                    .or_else(|| o.get("api_key"))
                    .or_else(|| o.get("token"))
            })
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // models 是字典，取第一个 key 作为 default_model
        let default_model = cfg
            .get("models")
            .and_then(|m| m.as_object())
            .and_then(|map| map.keys().next())
            .map(|s| s.to_string());

        // OpenCode 用 npm 包名标识协议（如 @ai-sdk/openai-compatible / @ai-sdk/anthropic）
        let npm_pkg = cfg.get("npm").and_then(|s| s.as_str()).unwrap_or("");
        let protocol = if npm_pkg.contains("anthropic") || base_url.contains("/anthropic") {
            "anthropic"
        } else if npm_pkg.contains("google") || npm_pkg.contains("gemini") {
            "gemini"
        } else {
            "openai"
        };

        let mut warnings = Vec::new();
        if api_key.is_none() {
            warnings.push("apiKey 字段缺失，导入后需手动配置".into());
        }

        out.push(ChannelCandidate {
            id: format!("opencode:{}", key),
            name: key.clone(),
            source_agent_id: agent_id.to_string(),
            source_platform: "opencode".to_string(),
            base_url,
            api_key,
            protocol: protocol.to_string(),
            default_model,
            provider_kind: "custom".to_string(),
            already_exists: false,
            warnings,
        });
    }
    out
}
