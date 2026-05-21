//! 协议归一化的 trait + NormalizedRequest 数据结构
//!
//! 不变量：`normalize → security pipeline → denormalize → forward`
//! 安全引擎只见过 NormalizedRequest；新加协议只需写一对 normalizer/denormalizer。

use super::formats::LlmFormat;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait RequestNormalizer: Send + Sync {
    fn format(&self) -> LlmFormat;
    fn normalize(&self, raw: &Value, headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError>;
    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError>;
}

#[derive(Debug, thiserror::Error)]
pub enum NormalizeError {
    #[error("missing field: {0}")]
    MissingField(&'static str),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRequest {
    pub model: String,
    pub messages: Vec<NormMessage>,
    pub tools: Vec<ToolDefinition>,
    pub stream: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub original_format: LlmFormat,
    pub original_body: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormMessage {
    pub role: Role,
    /// 纯文本用于安全检查
    pub text_content: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// 提取所有用户/系统文本到一个串，喂给 L2 安全检查。
pub fn collect_all_text(req: &NormalizedRequest) -> String {
    req.messages
        .iter()
        .filter(|m| matches!(m.role, Role::User | Role::System))
        .map(|m| m.text_content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// 简化 HeaderMap 抽象（避免依赖 http crate；W5 接 hudsucker 后替换为 http::HeaderMap）
pub mod http_like {
    use std::collections::HashMap;
    pub type HeaderMap = HashMap<String, String>;
}
