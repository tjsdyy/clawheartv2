//! MCP 拦截代理模块（W11.1 骨架）
//!
//! 子模块：
//!   - jsonrpc       JSON-RPC 2.0 类型与解析
//!   - interceptor   McpInterceptor trait + McpVerdict
//!   - stdio_bridge  stdio 双向转发桥
//!
//! 使用：
//!   1. 实现 McpInterceptor（见 security/mcp_check.rs::ClawHeartMcpInterceptor）
//!   2. 构造 StdioBridge::new(cfg, Arc::new(interceptor))
//!   3. 调 bridge.run(stdin, stdout) 接管 client 的 stdio
//!
//! W11.2 在 main.rs 加 `clawheart mcp-proxy` CLI 子命令时使用此模块。

pub mod interceptor;
pub mod jsonrpc;
pub mod stdio_bridge;

pub use interceptor::{CheckCtx, McpInterceptor, McpSession, McpVerdict, NoopMcpInterceptor};
pub use jsonrpc::{
    build_error_response, parse_line, serialize_message, McpMessage, McpNotification, McpRequest,
    McpResponse, RpcError, RpcId,
};
pub use stdio_bridge::{
    process_client_to_server, process_server_to_client, BridgeConfig, ProcessOutcome,
    StdioBridge,
};
