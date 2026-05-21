//! W10：虚拟 Key 反查与请求改写
//!
//! 当 hudsucker 拦截到带 `Authorization: Bearer sk-claw-xxx` 的请求时：
//! 1. 提取虚拟 key
//! 2. 反查 provider_profiles 表 → 拿到 Profile（base_url / credential_ref / protocol）
//! 3. 从 OS Keychain 取出真实 API key
//! 4. 改写请求：替换 Authorization + Host + URI 到真实 upstream
//!
//! 设计要点：
//! - 本模块完全 stateless，不依赖 hudsucker；可独立单测
//! - 错误均返回 ResolveError 枚举，handler 自行决定如何 fail-closed
//! - 真实 API key 从 Keychain 解密后立即放进 Authorization 头，不长期持有
//! - 与监控模式的 L0 协议归一化解耦（路由发生在归一化之前）

use crate::state::AppState;
use serde::Serialize;

const VIRTUAL_KEY_PREFIX: &str = "sk-claw-";

#[derive(Debug, Clone, Serialize)]
pub struct UpstreamRouting {
    pub profile_id: String,
    pub virtual_key: String,
    pub real_authorization: String, // "Bearer sk-or-..." 完整 header 值
    pub upstream_base_url: String,
    pub protocol: String,
    /// 可选：附加请求头（来自 profile.headers_json）
    pub extra_headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ResolveError {
    NotAVirtualKey,           // 不是 sk-claw- 前缀，直接原样转发
    UnknownVirtualKey,        // 前缀对但 DB 里查不到 → fail-closed
    StorageDisabled,          // storage feature 未启用
    KeychainFailure(String),  // Keychain 读取失败
    DbFailure(String),
    ProfileDisabled,
    NoCredential,
}

#[derive(Debug)]
pub enum RouteOutcome {
    /// 原样转发（虚拟 key 不存在，或非 ClawHeart 路由的请求）
    PassThrough,
    /// 改写为指向真实 upstream
    Rewrite(UpstreamRouting),
    /// 拒绝（已知虚拟 key 但凭据缺失等）
    Reject(ResolveError),
}

/// 提取 Authorization 头中的 Bearer token（不区分大小写）
pub fn extract_bearer_token(authorization_header: Option<&str>) -> Option<&str> {
    let raw = authorization_header?;
    let s = raw.trim();
    // 兼容 "Bearer xxx" / "bearer xxx"
    if s.len() >= 7 && s[..7].eq_ignore_ascii_case("Bearer ") {
        Some(s[7..].trim())
    } else {
        None
    }
}

pub fn is_virtual_key(token: &str) -> bool {
    token.starts_with(VIRTUAL_KEY_PREFIX)
}

/// 主入口：给定 Authorization header 值，决定如何路由
///
/// 注意：state 参数即便没有 storage feature 也保留签名，以便 handler 调用方一致。
pub fn route_for_authorization(
    state: &AppState,
    authorization_header: Option<&str>,
) -> RouteOutcome {
    let token = match extract_bearer_token(authorization_header) {
        Some(t) => t,
        None => return RouteOutcome::PassThrough, // 无 Bearer → 透传
    };

    if !is_virtual_key(token) {
        return RouteOutcome::PassThrough;
    }

    resolve_virtual_key(state, token)
}

/// 仅做反查：给定 sk-claw-xxx，找 Profile + 真实 key
pub fn resolve_virtual_key(state: &AppState, virtual_key: &str) -> RouteOutcome {
    #[cfg(feature = "storage")]
    {
        let db = match &state.db {
            Some(db) => db,
            None => return RouteOutcome::Reject(ResolveError::StorageDisabled),
        };

        let row = match crate::storage::queries::providers::get_by_virtual_key(
            db,
            virtual_key,
        ) {
            Ok(Some(r)) => r,
            Ok(None) => return RouteOutcome::Reject(ResolveError::UnknownVirtualKey),
            Err(e) => {
                return RouteOutcome::Reject(ResolveError::DbFailure(e.to_string()))
            }
        };

        if !row.enabled {
            return RouteOutcome::Reject(ResolveError::ProfileDisabled);
        }

        let real_key = match crate::storage::keychain::fetch(&row.credential_ref) {
            Ok(Some(k)) => k,
            Ok(None) => return RouteOutcome::Reject(ResolveError::NoCredential),
            Err(e) => return RouteOutcome::Reject(ResolveError::KeychainFailure(e)),
        };

        // 解析 headers_json
        let mut extra_headers = Vec::new();
        if let Some(json_str) = &row.headers_json {
            if let Ok(serde_json::Value::Object(map)) =
                serde_json::from_str::<serde_json::Value>(json_str)
            {
                for (k, v) in map {
                    if let serde_json::Value::String(s) = v {
                        extra_headers.push((k, s));
                    }
                }
            }
        }

        return RouteOutcome::Rewrite(UpstreamRouting {
            profile_id: row.id,
            virtual_key: row.virtual_key,
            real_authorization: format!("Bearer {}", real_key),
            upstream_base_url: row.base_url,
            protocol: row.protocol,
            extra_headers,
        });
    }

    #[cfg(not(feature = "storage"))]
    {
        let _ = (state, virtual_key);
        RouteOutcome::Reject(ResolveError::StorageDisabled)
    }
}

/// 工具：把 URI path 与 upstream_base_url 拼成完整目标 URL
pub fn build_upstream_url(upstream_base_url: &str, original_path_and_query: &str) -> String {
    let trimmed_base = upstream_base_url.trim_end_matches('/');
    let path = if original_path_and_query.starts_with('/') {
        original_path_and_query.to_string()
    } else {
        format!("/{}", original_path_and_query)
    };
    format!("{}{}", trimmed_base, path)
}

// ──────────────────────────────────────────────────────────────────
// 单元测试
// ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_basic() {
        assert_eq!(
            extract_bearer_token(Some("Bearer sk-claw-abc")),
            Some("sk-claw-abc")
        );
        assert_eq!(
            extract_bearer_token(Some("bearer sk-claw-abc")),
            Some("sk-claw-abc")
        );
        assert_eq!(extract_bearer_token(Some("Basic xxx")), None);
        assert_eq!(extract_bearer_token(None), None);
    }

    #[test]
    fn test_extract_bearer_whitespace() {
        assert_eq!(
            extract_bearer_token(Some("  Bearer   sk-claw-xyz   ")),
            Some("sk-claw-xyz")
        );
    }

    #[test]
    fn test_is_virtual_key() {
        assert!(is_virtual_key("sk-claw-abc123"));
        assert!(!is_virtual_key("sk-or-xxxxx"));
        assert!(!is_virtual_key("sk-ant-xxxxx"));
        assert!(!is_virtual_key(""));
    }

    #[test]
    fn test_build_upstream_url() {
        assert_eq!(
            build_upstream_url("https://openrouter.ai/api/v1", "/chat/completions"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            build_upstream_url("https://openrouter.ai/api/v1/", "/chat/completions"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            build_upstream_url("https://api.openai.com/v1", "chat/completions?foo=bar"),
            "https://api.openai.com/v1/chat/completions?foo=bar"
        );
    }
}
