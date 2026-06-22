use super::{SttError, Transcription};
use crate::config::schema::SttConfig;

/// STT Provider 统一接口
pub trait SttProvider: Send + Sync {
    /// 识别音频数据
    ///
    /// # 参数
    /// - `audio`: f32 PCM 采样数据，16kHz 单声道
    /// - `sample_rate`: 采样率
    fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<Transcription, SttError>;

    /// Provider 名称
    fn name(&self) -> &str;

    /// 是否已就绪（模型已加载等）
    fn is_ready(&self) -> bool;
}

/// 根据配置创建 STT Provider
pub fn create_provider(config: &SttConfig) -> Box<dyn SttProvider> {
    match config.provider.as_str() {
        "whisper-local" => {
            Box::new(super::whisper_local::WhisperLocalProvider::new(&config.local))
        }
        "openai" => {
            Box::new(super::whisper_api::WhisperApiProvider::new(&config.api))
        }
        _ => {
            tracing::warn!("未知的 STT provider: {}, 回退到本地 Whisper", config.provider);
            Box::new(super::whisper_local::WhisperLocalProvider::new(&config.local))
        }
    }
}
