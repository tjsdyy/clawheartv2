//! Hermes 反向导入
//!
//! 配置文件：~/.hermes/config.yaml （YAML）
//! 多 provider 结构：`custom_providers: [{ name, base_url, api_key, model, models }, ...]`
//! 参考：cc-switch hermes_config.rs

use crate::agents::config_probe::home_dir;
use crate::agents::probes::ChannelCandidate;

pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(home) = home_dir() else { return vec![]; };

    // 主：~/.hermes/config.yaml
    let yaml_path = home.join(".hermes/config.yaml");
    let content = match std::fs::read_to_string(&yaml_path) {
        Ok(c) => c,
        Err(_) => match std::fs::read_to_string(home.join(".hermes/config.yml")) {
            Ok(c) => c,
            Err(_) => return vec![],
        },
    };

    let v: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    // 兜底：从 .env 拿可能的全局 API key
    let env_api_key: Option<String> = std::fs::read_to_string(home.join(".hermes/.env"))
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with('#') {
                    continue;
                }
                if let Some(eq) = t.find('=') {
                    let key = t[..eq].trim();
                    if key == "OPENAI_API_KEY" || key == "ANTHROPIC_API_KEY" {
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

    let mut out = Vec::new();

    // custom_providers 是数组
    if let Some(providers) = v.get("custom_providers").and_then(|p| p.as_sequence()) {
        for item in providers {
            let name = yaml_str(item.get("name")).unwrap_or_else(|| "provider".to_string());
            let base_url = yaml_str(item.get("base_url"))
                .or_else(|| yaml_str(item.get("baseUrl")))
                .or_else(|| yaml_str(item.get("endpoint")))
                .unwrap_or_default();
            if base_url.is_empty() {
                continue;
            }
            let api_key = yaml_str(item.get("api_key"))
                .or_else(|| yaml_str(item.get("apiKey")))
                .or_else(|| yaml_str(item.get("token")))
                .or_else(|| env_api_key.clone()); // fallback .env

            let default_model = yaml_str(item.get("model"));

            // 推断协议：根据 base_url 启发式
            let protocol = if base_url.contains("/anthropic")
                || base_url.contains("api.anthropic.com")
            {
                "anthropic"
            } else if base_url.contains("generativelanguage") {
                "gemini"
            } else {
                "openai"
            };

            let mut warnings = Vec::new();
            if api_key.is_none() {
                warnings
                    .push("api_key 字段缺失，导入后需手动配置（也可能在 ~/.hermes/.env 中）".into());
            }

            out.push(ChannelCandidate {
                id: format!("hermes:{}", name),
                name: name.clone(),
                source_agent_id: agent_id.to_string(),
                source_platform: "hermes".to_string(),
                base_url,
                api_key,
                protocol: protocol.to_string(),
                default_model,
                provider_kind: "custom".to_string(),
                already_exists: false,
                warnings,
            });
        }
    }

    // 兜底：单 model 顶层配置（无 custom_providers）
    if out.is_empty() {
        if let Some(model_section) = v.get("model") {
            let base_url = yaml_str(model_section.get("base_url"))
                .or_else(|| yaml_str(model_section.get("baseUrl")))
                .unwrap_or_default();
            if !base_url.is_empty() {
                let provider_name =
                    yaml_str(model_section.get("provider")).unwrap_or_else(|| "default".into());
                let default_model = yaml_str(model_section.get("default"));
                let protocol = if base_url.contains("anthropic") {
                    "anthropic"
                } else if base_url.contains("generativelanguage") {
                    "gemini"
                } else {
                    "openai"
                };
                let mut warnings = Vec::new();
                if env_api_key.is_none() {
                    warnings.push(
                        "未在 config.yaml 或 ~/.hermes/.env 找到 API key，导入后需手动配置".into(),
                    );
                }
                out.push(ChannelCandidate {
                    id: format!("hermes:{}", provider_name),
                    name: provider_name,
                    source_agent_id: agent_id.to_string(),
                    source_platform: "hermes".to_string(),
                    base_url,
                    api_key: env_api_key.clone(),
                    protocol: protocol.to_string(),
                    default_model,
                    provider_kind: "custom".to_string(),
                    already_exists: false,
                    warnings,
                });
            }
        }
    }

    out
}

fn yaml_str(v: Option<&serde_yaml::Value>) -> Option<String> {
    v?.as_str().filter(|s| !s.is_empty()).map(|s| s.to_string())
}
