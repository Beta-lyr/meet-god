use serde::{Deserialize, Serialize};

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub stt: SttConfig,
    pub llm: LlmConfig,
    pub ui: UiConfig,
    pub hotkeys: HotkeyConfig,
    pub profile: ProfileConfig,
}

/// 音频捕获配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    /// 音频设备，"default" 表示系统默认
    pub device: String,
    /// 采样率（Whisper 要求 16kHz）
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 是否启用 VAD 静音检测
    pub vad_enabled: bool,
    /// VAD 静音阈值 (0.0-1.0)
    pub vad_threshold: f32,
}

/// STT 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SttConfig {
    /// 提供商: whisper-local | openai | xunfei
    pub provider: String,
    /// 本地 Whisper 配置
    pub local: WhisperLocalConfig,
    /// 云端 API 配置
    pub api: SttApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WhisperLocalConfig {
    /// 模型大小: tiny | base | small | medium
    pub model: String,
    /// 强制语言，留空自动检测
    pub language: String,
    /// 推理设备: cpu | cuda
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SttApiConfig {
    pub api_key: String,
    pub api_url: String,
    /// 模型名称
    pub model: String,
    /// 超时时间(ms)
    pub timeout_ms: u64,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    pub primary: LlmProviderConfig,
    pub fallback: LlmProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmProviderConfig {
    /// API 格式: openai | anthropic
    /// openai: 兼容 OpenAI /v1/chat/completions 格式（适用于 DeepSeek、通义千问、智谱、Ollama 等）
    /// anthropic: Anthropic /v1/messages 格式（Claude 系列）
    pub provider: String,
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    /// 超时时间(ms)
    pub timeout_ms: u64,
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// 窗口透明度 (0.1-1.0)
    pub opacity: f64,
    /// 字体大小
    pub font_size: u32,
    /// 始终置顶
    pub always_on_top: bool,
    /// 主题: dark | light
    pub theme: String,
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HotkeyConfig {
    /// 显示/隐藏答案窗
    pub toggle_visibility: String,
    /// 静音/取消静音
    pub toggle_mute: String,
    /// 紧急退出
    pub emergency_exit: String,
}

/// 用户资料配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileConfig {
    /// 简历内容
    pub resume: String,
    /// 目标岗位 JD
    pub job_description: String,
    /// 自定义 System Prompt
    pub custom_prompt: String,
}

// ========== Default 实现 ==========

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            stt: SttConfig::default(),
            llm: LlmConfig::default(),
            ui: UiConfig::default(),
            hotkeys: HotkeyConfig::default(),
            profile: ProfileConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device: "default".to_string(),
            sample_rate: 16000,
            channels: 1,
            vad_enabled: true,
            vad_threshold: 0.5,
        }
    }
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            provider: "whisper-local".to_string(),
            local: WhisperLocalConfig::default(),
            api: SttApiConfig::default(),
        }
    }
}

impl Default for WhisperLocalConfig {
    fn default() -> Self {
        Self {
            model: "base".to_string(),
            language: "zh".to_string(),
            device: "cpu".to_string(),
        }
    }
}

impl Default for SttApiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_url: String::new(),
            model: "whisper-1".to_string(),
            timeout_ms: 10000,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            primary: LlmProviderConfig::default(),
            fallback: LlmProviderConfig::default(),
        }
    }
}

impl Default for LlmProviderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: String::new(),
            api_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
            max_tokens: 1024,
            timeout_ms: 15000,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            opacity: 0.9,
            font_size: 14,
            always_on_top: true,
            theme: "dark".to_string(),
        }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            toggle_visibility: "Ctrl+Shift+H".to_string(),
            toggle_mute: "Ctrl+Shift+M".to_string(),
            emergency_exit: "Ctrl+Shift+Q".to_string(),
        }
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            resume: String::new(),
            job_description: String::new(),
            custom_prompt: String::new(),
        }
    }
}
