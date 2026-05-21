//! MCP 拦截 trait 与判决枚举
//!
//! 设计：两个独立方法，分别处理两个方向。每次返回 McpVerdict 决定如何处置。
//!
//! 关键不变量：
//!   - check_request 在向 upstream 转发前调用
//!   - check_response 在向 client 回传前调用
//!   - 任一返回 Block，对应一方收到 RpcError 响应/被拦截
//!   - Strip 仅在 check_response 有意义：替换内容（如脱敏）但不改判决

use super::jsonrpc::{McpRequest, McpResponse};

/// 拦截判决
#[derive(Debug, Clone)]
pub enum McpVerdict {
    /// 透传
    Allow,
    /// 拦截：连请求都不发到 upstream（或回错给 client）
    Block { reason: String, layer: String },
    /// 允许，但写一条警告事件
    Warn { reason: String, layer: String },
    /// 仅用于响应方向：用 sanitized_text 替换原 text 内容
    Strip { sanitized_text: String, reason: String },
}

impl McpVerdict {
    pub fn is_block(&self) -> bool {
        matches!(self, McpVerdict::Block { .. })
    }
}

/// 一次会话上下文 —— 跨 request/response 共享（如攻击链状态）
#[derive(Debug, Clone)]
pub struct McpSession {
    pub session_id: String,
    pub server_id: String, // 比如 "filesystem" / "github"
}

/// 检查上下文：传给每次 check 调用，含会话信息 + 上游服务标识
pub struct CheckCtx<'a> {
    pub session: &'a McpSession,
}

/// MCP 拦截器接口
///
/// 实现者：
///   - ClawHeartMcpInterceptor（接 chain_detector / DLP / injection）
///   - NoopMcpInterceptor（透传，用于测试）
pub trait McpInterceptor: Send + Sync {
    /// client → server 方向：扫工具参数 / 资源 URI / DLP
    fn check_request(&self, ctx: &CheckCtx<'_>, req: &McpRequest) -> McpVerdict;

    /// server → client 方向：扫工具结果 / 工具描述漂移 / injection
    fn check_response(
        &self,
        ctx: &CheckCtx<'_>,
        req: &McpRequest,
        resp: &McpResponse,
    ) -> McpVerdict;
}

/// 透传实现，用于本地测试 / 用户禁用拦截时
pub struct NoopMcpInterceptor;

impl McpInterceptor for NoopMcpInterceptor {
    fn check_request(&self, _ctx: &CheckCtx<'_>, _req: &McpRequest) -> McpVerdict {
        McpVerdict::Allow
    }
    fn check_response(
        &self,
        _ctx: &CheckCtx<'_>,
        _req: &McpRequest,
        _resp: &McpResponse,
    ) -> McpVerdict {
        McpVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::mcp::jsonrpc::{RpcId, McpRequest};

    #[test]
    fn noop_always_allows() {
        let interceptor = NoopMcpInterceptor;
        let session = McpSession {
            session_id: "s1".into(),
            server_id: "filesystem".into(),
        };
        let ctx = CheckCtx { session: &session };
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: RpcId::Number(1),
            method: "tools/list".into(),
            params: None,
        };
        assert!(!interceptor.check_request(&ctx, &req).is_block());
    }
}
