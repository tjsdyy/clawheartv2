//! 多协议 token / 用量提取
//!
//! 各家响应格式不同；统一抽到 TokenUsage 喂给预算 + 日志 + 同步。

use super::formats::LlmFormat;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
}

pub fn extract(format: LlmFormat, response: &Value) -> Option<TokenUsage> {
    match format {
        LlmFormat::OpenAI => {
            let u = response.get("usage")?;
            Some(TokenUsage {
                input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                ..Default::default()
            })
        }
        LlmFormat::Claude => {
            let u = response.get("usage")?;
            Some(TokenUsage {
                input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                cache_read_tokens: u.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                cache_creation_tokens: u.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            })
        }
        LlmFormat::OpenAIResponses => {
            let u = response.get("usage")?;
            Some(TokenUsage {
                input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                ..Default::default()
            })
        }
        LlmFormat::Gemini => {
            let u = response.get("usageMetadata")?;
            Some(TokenUsage {
                input_tokens: u.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: u.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                ..Default::default()
            })
        }
        LlmFormat::Ollama => {
            Some(TokenUsage {
                input_tokens: response.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: response.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                ..Default::default()
            })
        }
        LlmFormat::Unknown => None,
    }
}
