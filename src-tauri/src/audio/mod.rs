pub mod capture;
pub mod vad;

use serde::{Deserialize, Serialize};

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
