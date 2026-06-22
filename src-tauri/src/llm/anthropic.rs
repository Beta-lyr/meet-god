use super::{ChatConfig, LlmError, Message, Role, StreamChunk};
use super::provider::LlmProvider;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic API Provider
///
/// 支持 Anthropic /v1/messages 格式（Claude 系列模型）
pub struct AnthropicProvider {
    api_key: String,
    api_url: String,
    model: String,
    timeout_ms: u64,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

impl AnthropicProvider {
    pub fn new(config: &super::super::config::schema::LlmProviderConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_url: config.api_url.trim_end_matches('/').to_string(),
            model: config.model.clone(),
            timeout_ms: config.timeout_ms,
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn chat(&self, messages: &[Message], config: &ChatConfig) -> Result<String, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::ConfigError("API Key 未配置".to_string()));
        }

        // 分离 system prompt 和对话消息
        let system = if !config.system_prompt.is_empty() {
            Some(config.system_prompt.clone())
        } else {
            None
        };

        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "user".to_string(), // 不会走到这里
                },
                content: m.content.clone(),
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: config.max_tokens,
            system,
            messages: anthropic_messages,
            temperature: Some(config.temperature),
        };

        let client = Client::builder()
            .timeout(Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let url = format!("{}/v1/messages", self.api_url);

        let response = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let result: AnthropicResponse = response
            .json()
            .map_err(|e| LlmError::ApiError(format!("解析响应失败: {}", e)))?;

        result
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| LlmError::ApiError("响应内容为空".to_string()))
    }

    fn chat_stream(&self, messages: &[Message], config: &ChatConfig) -> Result<Vec<StreamChunk>, LlmError> {
        // Anthropic 流式 API 暂用同步替代
        let text = self.chat(messages, config)?;
        Ok(vec![StreamChunk {
            content: text,
            done: true,
        }])
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}
