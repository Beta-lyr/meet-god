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
///
/// 仅支持两种 API 格式：
/// - openai: 兼容 OpenAI /v1/chat/completions（适用于绝大多数国内外模型）
/// - anthropic: Anthropic /v1/messages（Claude 系列）
pub fn create_provider(config: &super::super::config::schema::LlmProviderConfig) -> Box<dyn LlmProvider> {
    match config.provider.as_str() {
        "anthropic" => {
            Box::new(super::anthropic::AnthropicProvider::new(config))
        }
        _ => {
            // 默认使用 OpenAI 兼容格式
            Box::new(super::openai::OpenAiProvider::new(config))
        }
    }
}
