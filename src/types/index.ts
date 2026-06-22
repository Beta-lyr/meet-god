/// 音频设备信息
export interface AudioDeviceInfo {
  name: string;
  sample_rate: number;
  channels: number;
}

/// 系统信息
export interface SystemInfo {
  os: string;
  os_version: string;
  arch: string;
}

/// 应用配置
export interface AppConfig {
  audio: AudioConfig;
  stt: SttConfig;
  llm: LlmConfig;
  ui: UiConfig;
  hotkeys: HotkeyConfig;
  profile: ProfileConfig;
}

export interface AudioConfig {
  device: string;
  sample_rate: number;
  channels: number;
  vad_enabled: boolean;
  vad_threshold: number;
}

export interface SttConfig {
  provider: "whisper-local" | "openai" | "xunfei";
  local: WhisperLocalConfig;
  api: SttApiConfig;
}

export interface WhisperLocalConfig {
  model: "tiny" | "base" | "small" | "medium";
  language: string;
  device: "cpu" | "cuda";
}

export interface SttApiConfig {
  api_key: string;
  api_url: string;
  model: string;
  timeout_ms: number;
}

export interface LlmConfig {
  primary: LlmProviderConfig;
  fallback: LlmProviderConfig;
}

export interface LlmProviderConfig {
  provider: string;
  api_key: string;
  api_url: string;
  model: string;
  temperature: number;
  max_tokens: number;
  timeout_ms: number;
}

export interface UiConfig {
  opacity: number;
  font_size: number;
  always_on_top: boolean;
  theme: "dark" | "light";
}

export interface HotkeyConfig {
  toggle_visibility: string;
  toggle_mute: string;
  emergency_exit: string;
}

export interface ProfileConfig {
  resume: string;
  job_description: string;
  custom_prompt: string;
}

/// Whisper 模型状态
export interface WhisperModelStatus {
  model: string;
  exists: boolean;
  path: string;
  download_url: string;
}

/// 管线事件（从后端推送）
export interface PipelineEvent {
  type: "Transcription" | "AnswerChunk" | "StateChange" | "Error";
  text?: string;
  confidence?: number;
  latency_ms?: number;
  content?: string;
  done?: boolean;
  state?: string;
  message?: string;
}

/// 答案条目
export interface AnswerEntry {
  id: string;
  question: string;
  answer: string;
  timestamp: number;
  latency_ms: number;
}

/// 管线状态
export interface PipelineStatus {
  running: boolean;
  audio_state: string;
  stt_provider: string;
  llm_provider: string;
}
