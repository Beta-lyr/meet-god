use super::{SttError, Transcription};
use crate::config::schema::WhisperLocalConfig;
use std::path::PathBuf;
use std::time::Instant;

/// 本地 Whisper STT Provider
///
/// 使用 whisper-rs (Whisper.cpp Rust 绑定) 进行本地语音识别。
/// 模型文件存储在应用数据目录。
///
/// 需要启用 `whisper-local` feature（需要安装 LLVM/Clang）
pub struct WhisperLocalProvider {
    model_path: PathBuf,
    language: String,
}

impl WhisperLocalProvider {
    pub fn new(config: &WhisperLocalConfig) -> Self {
        let model_path = Self::model_path(&config.model);
        let language = config.language.clone();

        tracing::info!(
            "Whisper 本地模型: {}, 路径: {:?}, 语言: {}",
            config.model,
            model_path,
            language
        );

        Self {
            model_path,
            language,
        }
    }

    /// 获取模型文件路径
    /// 存储在 %APPDATA%/meet-god/models/ 目录下
    fn model_path(model_name: &str) -> PathBuf {
        let app_data = std::env::var("APPDATA")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))
            .unwrap_or_else(|_| ".".to_string());

        PathBuf::from(app_data)
            .join("meet-god")
            .join("models")
            .join(format!("ggml-{}.bin", model_name))
    }

    /// 模型文件是否存在
    pub fn model_exists(&self) -> bool {
        self.model_path.exists()
    }

    /// 获取模型文件路径（实例方法）
    pub fn get_model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// 获取模型下载 URL
    pub fn model_download_url(model_name: &str) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model_name
        )
    }

    /// 使用 whisper-rs 进行推理（仅在启用 whisper-local feature 时可用）
    #[cfg(feature = "whisper-local")]
    fn run_inference(&self, audio: &[f32]) -> Result<String, SttError> {
        use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

        let ctx = WhisperContext::new_with_params(
            self.model_path.as_path(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| SttError::ModelLoadError(e.to_string()))?;

        let mut state = ctx
            .create_state()
            .map_err(|e| SttError::RecognitionError(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(&self.language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);

        state
            .full(params, audio)
            .map_err(|e| SttError::RecognitionError(e.to_string()))?;

        let num_segments = state.full_n_segments();
        let mut text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(segment_text) = segment.to_str() {
                    text.push_str(segment_text);
                }
            }
        }

        Ok(text)
    }
}

impl super::provider::SttProvider for WhisperLocalProvider {
    fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<Transcription, SttError> {
        let start = Instant::now();

        if !self.model_exists() {
            return Err(SttError::ModelLoadError(format!(
                "模型文件不存在: {:?}\n请先下载模型文件。",
                self.model_path
            )));
        }

        if sample_rate != 16000 {
            return Err(SttError::ConfigError(format!(
                "Whisper 要求 16kHz 采样率，当前: {}Hz",
                sample_rate
            )));
        }

        let _duration_ms = (audio.len() as f64 / sample_rate as f64 * 1000.0) as u64;

        // 根据 feature 选择推理方式
        #[cfg(feature = "whisper-local")]
        {
            let text = self.run_inference(audio)?;
            let latency_ms = start.elapsed().as_millis() as u64;
            return Ok(Transcription {
                text,
                confidence: 0.9,
                language: self.language.clone(),
                duration_ms: _duration_ms,
                latency_ms,
            });
        }

        #[cfg(not(feature = "whisper-local"))]
        {
            Err(SttError::ConfigError(
                "本地 Whisper 未启用。请安装 LLVM/Clang 后使用 --features whisper-local 编译。\n\
                 或者切换到云端 STT（设置 → STT Provider → openai）"
                    .to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        "whisper-local"
    }

    fn is_ready(&self) -> bool {
        self.model_exists()
    }
}
