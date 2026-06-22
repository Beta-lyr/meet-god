pub mod capture;
pub mod vad;

use serde::{Deserialize, Serialize};

/// 音频片段，从 WASAPI Loopback 捕获的 PCM 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    /// PCM 采样数据 (f32, 16kHz, 单声道)
    pub samples: Vec<f32>,
    /// 采样率
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 时间戳 (ms)
    pub timestamp_ms: u64,
}

/// 音频捕获状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureState {
    /// 未启动
    Idle,
    /// 正在捕获
    Running,
    /// 已暂停（静音）
    Muted,
    /// 出错
    Error,
}
