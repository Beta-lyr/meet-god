pub mod window;

use serde::{Deserialize, Serialize};

/// 隐蔽模式状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StealthState {
    /// 正常显示
    Visible,
    /// 已隐藏（用户手动隐藏）
    Hidden,
    /// 不可截图模式（对屏幕捕获不可见）
    Invisible,
}
