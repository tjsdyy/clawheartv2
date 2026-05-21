//! Host → (Provider, LlmFormat) 映射 — ~30 条已知端点
//!
//! 不维护 800+ 模型枚举；只用 host 推断协议族，模型 ID 从 body 提取。

use super::formats::LlmFormat;

#[derive(Debug, Clone, Copy)]
pub struct ProviderInfo {
    pub provider: &'static str,
    pub format: LlmFormat,
}

pub fn lookup_host(host: &str) -> Option<ProviderInfo> {
    let h = host.to_lowercase();
    Some(match h.as_str() {
        // Anthropic
        "api.anthropic.com" => ProviderInfo { provider: "anthropic", format: LlmFormat::Claude },

        // OpenAI
        "api.openai.com" => ProviderInfo { provider: "openai", format: LlmFormat::OpenAI },

        // Google
        h if h.contains("generativelanguage.googleapis.com")
            || h.contains("aiplatform.googleapis.com")
            => ProviderInfo { provider: "google", format: LlmFormat::Gemini },

        // Mistral
        "api.mistral.ai" => ProviderInfo { provider: "mistral", format: LlmFormat::OpenAI },

        // Groq
        "api.groq.com" => ProviderInfo { provider: "groq", format: LlmFormat::OpenAI },

        // Together
        "api.together.xyz" => ProviderInfo { provider: "together", format: LlmFormat::OpenAI },

        // DeepSeek
        "api.deepseek.com" => ProviderInfo { provider: "deepseek", format: LlmFormat::OpenAI },

        // Moonshot
        "api.moonshot.cn" => ProviderInfo { provider: "moonshot", format: LlmFormat::OpenAI },

        // Zhipu
        "open.bigmodel.cn" => ProviderInfo { provider: "zhipu", format: LlmFormat::OpenAI },

        // Local Ollama
        "localhost" | "127.0.0.1" if h.contains(":11434") =>
            ProviderInfo { provider: "ollama", format: LlmFormat::Ollama },

        _ => return None,
    })
}
