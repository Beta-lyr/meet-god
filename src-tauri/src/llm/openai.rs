use super::{ChatConfig, LlmError, Message, StreamChunk};
use crate::config::schema::LlmProviderConfig;

/// OpenAI 兼容 API Provider
///
/// 支持所有兼容 OpenAI /v1/chat/completions 格式的 API：
/// - OpenAI
/// - DeepSeek
/// - 通义千问
/// - 智谱
/// - Ollama
pub struct OpenAiProvider {
    api_key: String,
    api_url: String,
    timeout_ms: u64,
}

impl OpenAiProvider {
    pub fn new(config: &LlmProviderConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_url: config.api_url.clone(),
            timeout_ms: config.timeout_ms,
        }
    }

    /// 构建请求 URL
    fn chat_url(&self) -> String {
        let base = self.api_url.trim_end_matches('/');
        format!("{}/chat/completions", base)
    }
}

impl super::provider::LlmProvider for OpenAiProvider {
    fn chat(&self, messages: &[Message], config: &ChatConfig) -> Result<String, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::ConfigError("API Key 未配置".to_string()));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let body = build_request_body(messages, config, false);

        let response = client
            .post(self.chat_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(LlmError::ApiError(format!("{}: {}", status, body)));
        }

        let result: serde_json::Value = response
            .json()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn chat_stream(
        &self,
        messages: &[Message],
        config: &ChatConfig,
    ) -> Result<Vec<StreamChunk>, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::ConfigError("API Key 未配置".to_string()));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let body = build_request_body(messages, config, true);

        let response = client
            .post(self.chat_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(LlmError::ApiError(format!("{}: {}", status, body)));
        }

        let text = response.text().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        let mut chunks = Vec::new();

        // 解析 SSE 流
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            if data == "[DONE]" {
                chunks.push(StreamChunk {
                    content: String::new(),
                    done: true,
                });
                break;
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                let content = val["choices"][0]["delta"]["content"]
                    .as_str()
                    .unwrap_or("");
                if !content.is_empty() {
                    chunks.push(StreamChunk {
                        content: content.to_string(),
                        done: false,
                    });
                }
            }
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// 构建请求体
fn build_request_body(
    messages: &[Message],
    config: &ChatConfig,
    stream: bool,
) -> serde_json::Value {
    let msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": match m.role {
                    super::Role::System => "system",
                    super::Role::User => "user",
                    super::Role::Assistant => "assistant",
                },
                "content": m.content,
            })
        })
        .collect();

    serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "stream": stream,
    })
}
