//! Ollama (`/api/chat`) normalizer

use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::*;
use serde_json::Value;

pub struct OllamaNormalizer;

impl RequestNormalizer for OllamaNormalizer {
    fn format(&self) -> LlmFormat { LlmFormat::Ollama }

    fn normalize(&self, raw: &Value, _headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError> {
        let model = raw.get("model").and_then(|v| v.as_str())
            .ok_or(NormalizeError::MissingField("model"))?.to_string();

        let messages = raw.get("messages").and_then(|m| m.as_array())
            .ok_or(NormalizeError::MissingField("messages"))?;

        let mut norm_msgs = Vec::new();
        for m in messages {
            let role = match m.get("role").and_then(|r| r.as_str()).unwrap_or("user") {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                _ => Role::User,
            };
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
            norm_msgs.push(NormMessage {
                role, text_content: content, tool_calls: vec![], tool_results: vec![],
            });
        }

        Ok(NormalizedRequest {
            model, messages: norm_msgs, tools: vec![],
            stream: raw.get("stream").and_then(|s| s.as_bool()).unwrap_or(false),
            max_tokens: None,
            temperature: raw.pointer("/options/temperature").and_then(|v| v.as_f64()),
            original_format: LlmFormat::Ollama,
            original_body: raw.clone(),
        })
    }

    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError> {
        Ok(norm.original_body.clone())
    }
}
