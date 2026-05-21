//! 路由策略 — DIRECT / GATEWAY / MAPPING
//!
//! 决定一个出站请求最终去向：
//!   - DIRECT：直接走真实上游 host（默认）
//!   - GATEWAY：转发到用户配置的 OpenClaw 网关或私有中转
//!   - MAPPING：按 llm_mappings 表把 source host 改写到 target host（v1 兼容）

use super::formats::LlmFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub mode: RouteMode,
    pub target_host: String,
    pub target_path_prefix: Option<String>,
    pub upstream_format: LlmFormat,
    pub auth_header: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RouteMode {
    Direct,
    Gateway,
    Mapping,
}

#[derive(Debug, Clone, Default)]
pub struct Router {
    pub mappings: Vec<HostMapping>,
    pub gateway: Option<GatewayConfig>,
}

#[derive(Debug, Clone)]
pub struct HostMapping {
    pub source_host: String,
    pub target_host: String,
    pub target_path: Option<String>,
    pub format: LlmFormat,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub url: String,
    pub auth_header: Option<String>,
    /// 把所有支持的格式都转发到 gateway（gateway 自行处理）
    pub catch_all: bool,
}

impl Router {
    pub fn route(&self, host: &str, detected_format: LlmFormat) -> RouteDecision {
        // 1. Mapping 优先
        if let Some(m) = self.mappings.iter().find(|m| m.source_host.eq_ignore_ascii_case(host)) {
            return RouteDecision {
                mode: RouteMode::Mapping,
                target_host: m.target_host.clone(),
                target_path_prefix: m.target_path.clone(),
                upstream_format: m.format,
                auth_header: None,
            };
        }

        // 2. Gateway catch-all
        if let Some(gw) = &self.gateway {
            if gw.catch_all {
                return RouteDecision {
                    mode: RouteMode::Gateway,
                    target_host: gw.url.clone(),
                    target_path_prefix: None,
                    upstream_format: detected_format,
                    auth_header: gw.auth_header.clone(),
                };
            }
        }

        // 3. 默认 Direct
        RouteDecision {
            mode: RouteMode::Direct,
            target_host: host.into(),
            target_path_prefix: None,
            upstream_format: detected_format,
            auth_header: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_direct() {
        let r = Router::default();
        let d = r.route("api.anthropic.com", LlmFormat::Claude);
        assert_eq!(d.mode, RouteMode::Direct);
        assert_eq!(d.target_host, "api.anthropic.com");
    }

    #[test]
    fn mapping_wins() {
        let r = Router {
            mappings: vec![HostMapping {
                source_host: "api.anthropic.com".into(),
                target_host: "claude.myproxy.com".into(),
                target_path: None,
                format: LlmFormat::Claude,
            }],
            ..Default::default()
        };
        let d = r.route("api.anthropic.com", LlmFormat::Claude);
        assert_eq!(d.mode, RouteMode::Mapping);
        assert_eq!(d.target_host, "claude.myproxy.com");
    }
}
