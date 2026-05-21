//! OpenAI Responses API (`/v1/responses`) normalizer — Codex

use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::*;
use serde_json::Value;

pub struct ResponsesNormalizer;

impl RequestNormalizer for ResponsesNormalizer {
    fn format(&self) -> LlmFormat { LlmFormat::OpenAIResponses }

    fn normalize(&self, raw: &Value, _headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError> {
        let model = raw.get("model").and_then(|v| v.as_str())
            .ok_or(NormalizeError::MissingField("model"))?.to_string();

        let mut messages = Vec::new();
        // Responses API 的 input 可以是 string 或 array
        match raw.get("input") {
            Some(Value::String(s)) => messages.push(NormMessage {
                role: Role::User, text_content: s.clone(),
                tool_calls: vec![], tool_results: vec![],
            }),
            Some(Value::Array(arr)) => {
                for item in arr {
                    let role = match item.get("role").and_then(|r| r.as_str()).unwrap_or("user") {
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        "system" => Role::System,
                        _ => Role::User,
                    };
                    let content = item.get("content").and_then(|c| c.as_str())
                        .unwrap_or("").to_string();
                    messages.push(NormMessage {
                        role, text_content: content, tool_calls: vec![], tool_results: vec![],
                    });
                }
            }
            _ => return Err(NormalizeError::MissingField("input")),
        }

        Ok(NormalizedRequest {
            model, messages, tools: vec![],
            stream: raw.get("stream").and_then(|s| s.as_bool()).unwrap_or(false),
            max_tokens: raw.get("max_output_tokens").and_then(|v| v.as_u64()).map(|n| n as u32),
            temperature: raw.get("temperature").and_then(|v| v.as_f64()),
            original_format: LlmFormat::OpenAIResponses,
            original_body: raw.clone(),
        })
    }

    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError> {
        Ok(norm.original_body.clone())
    }
}
