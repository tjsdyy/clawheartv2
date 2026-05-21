//! LLM 协议格式枚举

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LlmFormat {
    /// OpenAI Chat Completions
    OpenAI,
    /// OpenAI Responses API（Codex）
    OpenAIResponses,
    /// Anthropic Messages API（Claude Code）
    Claude,
    /// Google Gemini
    Gemini,
    /// Ollama
    Ollama,
    /// 未识别
    Unknown,
}

impl LlmFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmFormat::OpenAI          => "openai",
            LlmFormat::OpenAIResponses => "openai_responses",
            LlmFormat::Claude          => "claude",
            LlmFormat::Gemini          => "gemini",
            LlmFormat::Ollama          => "ollama",
            LlmFormat::Unknown         => "unknown",
        }
    }
}
