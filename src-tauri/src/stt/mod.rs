pub mod provider;
pub mod whisper_local;
pub mod whisper_api;

use serde::{Deserialize, Serialize};

/// STT 识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    /// 识别文本
    pub text: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 识别语言
    pub language: String,
    /// 音频时长 (ms)
    pub duration_ms: u64,
    /// 识别耗时 (ms)
    pub latency_ms: u64,
}

/// STT 识别错误
#[derive(Debug, thiserror::Error)]
pub enum SttError {
    #[error("模型加载失败: {0}")]
    ModelLoadError(String),
    #[error("识别失败: {0}")]
    RecognitionError(String),
    #[error("网络错误: {0}")]
    NetworkError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
}
