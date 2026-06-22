use super::{ChatConfig, LlmError, Message, StreamChunk};

/// LLM Provider 统一接口
pub trait LlmProvider: Send + Sync {
    /// 同步聊天（等待完整响应）
    fn chat(&self, messages: &[Message], config: &ChatConfig) -> Result<String, LlmError>;

    /// 流式聊天（逐 token 返回）
    fn chat_stream(
        &self,
        messages: &[Message],
        config: &ChatConfig,
    ) -> Result<Vec<StreamChunk>, LlmError>;

    /// Provider 名称
    fn name(&self) -> &str;

    /// 是否已就绪
    fn is_ready(&self) -> bool;
}

/// 根据配置创建 LLM Provider
pub fn create_provider(config: &super::super::config::schema::LlmProviderConfig) -> Box<dyn LlmProvider> {
    match config.provider.as_str() {
        "openai" | "deepseek" | "qwen" | "zhipu" => {
            Box::new(super::openai::OpenAiProvider::new(config))
        }
        "ollama" => {
            // Ollama 兼容 OpenAI 格式
            Box::new(super::openai::OpenAiProvider::new(config))
        }
        _ => {
            tracing::warn!("未知的 LLM provider: {}, 回退到 OpenAI 兼容模式", config.provider);
            Box::new(super::openai::OpenAiProvider::new(config))
        }
    }
}
