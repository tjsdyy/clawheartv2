//! OpenAI Chat Completions normalizer

use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::*;
use serde_json::Value;

pub struct OpenAiNormalizer;

impl RequestNormalizer for OpenAiNormalizer {
    fn format(&self) -> LlmFormat { LlmFormat::OpenAI }

    fn normalize(&self, raw: &Value, _headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError> {
        let model = raw.get("model").and_then(|v| v.as_str())
            .ok_or(NormalizeError::MissingField("model"))?.to_string();

        let messages = raw.get("messages").and_then(|m| m.as_array())
            .ok_or(NormalizeError::MissingField("messages"))?;

        let mut norm_messages = Vec::new();
        for msg in messages {
            let role_str = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let role = match role_str {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                "tool" => Role::Tool,
                _ => Role::User,
            };
            let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
            norm_messages.push(NormMessage {
                role, text_content: content,
                tool_calls: vec![], tool_results: vec![],
            });
        }

        Ok(NormalizedRequest {
            model,
            messages: norm_messages,
            tools: vec![],
            stream: raw.get("stream").and_then(|s| s.as_bool()).unwrap_or(false),
            max_tokens: raw.get("max_tokens").and_then(|v| v.as_u64()).map(|n| n as u32),
            temperature: raw.get("temperature").and_then(|v| v.as_f64()),
            original_format: LlmFormat::OpenAI,
            original_body: raw.clone(),
        })
    }

    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError> {
        // OpenAI 是源格式 → 直接返回 original_body
        Ok(norm.original_body.clone())
    }
}
