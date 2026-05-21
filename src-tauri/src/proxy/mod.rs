//! 代理引擎（L0–L1）— hudsucker MITM 起转点
//!
//! W5 起 `proxy_real` feature 接入 hudsucker；当前 alpha 阶段为骨架 + 协议归一化纯逻辑。

pub mod ca_manager;
pub mod circuit_breaker;
pub mod credential_router;
pub mod cross_request;
pub mod fetch_server;
pub mod format_detector;
pub mod formats;
pub mod handler;
pub mod mcp;
pub mod normalizer;
pub mod normalizers;
pub mod pipeline;
pub mod provider_registry;
pub mod route;
pub mod security_check;
pub mod server;
pub mod streaming;
pub mod tls;
pub mod usage_extractor;
