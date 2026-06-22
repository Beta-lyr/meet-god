pub mod provider;
pub mod openai;

use serde::{Deserialize, Serialize};

/// LLM 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// 聊天请求配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: String,
}

/// LLM 流式响应片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// 文本片段
    pub content: String,
    /// 是否为最后一个片段
    pub done: bool,
}

/// LLM 错误
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("网络错误: {0}")]
    NetworkError(String),
    #[error("API 错误: {0}")]
    ApiError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("超时")]
    Timeout,
}
