//! OpenEva 反向导入
//!
//! 解析 ~/.openeva/tool-providers.json （多 provider 数组结构）+ 兜底读 settings.json

use crate::agents::config_probe::home_dir;
use crate::agents::probes::ChannelCandidate;

pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(home) = home_dir() else { return vec![]; };

    let mut out = Vec::new();

    // 1. 主源：~/.openeva/tool-providers.json
    let tp_path = home.join(".openeva/tool-providers.json");
    if let Ok(content) = std::fs::read_to_string(&tp_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            extract_from_value(&v, agent_id, &mut out);
        }
    }

    // 2. 兜底：~/.openeva/settings.json 中的 providers 字段（如有）
    if out.is_empty() {
        let s_path = home.join(".openeva/settings.json");
        if let Ok(content) = std::fs::read_to_string(&s_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                // 常见路径：providers / llm.providers / tools.providers
                let candidates_paths = [
                    "/providers",
                    "/llm/providers",
                    "/tools/providers",
                    "/models/providers",
                ];
                for p in candidates_paths {
                    if let Some(node) = v.pointer(p) {
                        extract_from_value(node, agent_id, &mut out);
                        if !out.is_empty() {
                            break;
                        }
                    }
                }
            }
        }
    }

    out
}

/// 尝试从 JSON 节点提取候选：兼容数组（[{name, baseUrl, ...}, ...]）和字典（{key: {...}}）
fn extract_from_value(v: &serde_json::Value, agent_id: &str, out: &mut Vec<ChannelCandidate>) {
    if let Some(arr) = v.as_array() {
        for (i, item) in arr.iter().enumerate() {
            let name = item
                .get("name")
                .or_else(|| item.get("id"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("provider_{}", i));
            if let Some(c) = build_candidate(&name, item, agent_id) {
                out.push(c);
            }
        }
    } else if let Some(obj) = v.as_object() {
        for (key, item) in obj {
            if let Some(c) = build_candidate(key, item, agent_id) {
                out.push(c);
            }
        }
    }
}

fn build_candidate(
    name: &str,
    cfg: &serde_json::Value,
    agent_id: &str,
) -> Option<ChannelCandidate> {
    let base_url = cfg
        .get("baseUrl")
        .or_else(|| cfg.get("base_url"))
        .or_else(|| cfg.get("endpoint"))
        .or_else(|| cfg.get("url"))
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())?
        .to_string();

    let api_key = cfg
        .get("apiKey")
        .or_else(|| cfg.get("api_key"))
        .or_else(|| cfg.get("token"))
        .or_else(|| cfg.get("key"))
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let default_model = cfg
        .get("model")
        .or_else(|| cfg.get("default_model"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let api_type = cfg
        .get("type")
        .or_else(|| cfg.get("protocol"))
        .or_else(|| cfg.get("api"))
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let protocol = if api_type.contains("anthropic") || base_url.contains("/anthropic") {
        "anthropic"
    } else if api_type.contains("gemini") || base_url.contains("generativelanguage") {
        "gemini"
    } else {
        "openai"
    };

    let mut warnings = Vec::new();
    if api_key.is_none() {
        warnings.push("未找到凭据字段，导入后需手动配置".into());
    }

    Some(ChannelCandidate {
        id: format!("openeva:{}", name),
        name: name.to_string(),
        source_agent_id: agent_id.to_string(),
        source_platform: "openeva".to_string(),
        base_url,
        api_key,
        protocol: protocol.to_string(),
        default_model,
        provider_kind: "custom".to_string(),
        already_exists: false,
        warnings,
    })
}
