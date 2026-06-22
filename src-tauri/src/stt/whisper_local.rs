use super::{SttError, Transcription};
use crate::config::schema::WhisperLocalConfig;
use std::io::Write;
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

    /// 下载模型文件（同步，阻塞当前线程）
    /// 返回下载的字节数
    pub fn download_model(model_name: &str) -> Result<u64, String> {
        let url = Self::model_download_url(model_name);
        let path = Self::model_path(model_name);

        // 创建目录
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        tracing::info!("开始下载模型: {} -> {:?}", url, path);

        // 下载文件
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("下载请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("下载失败，HTTP 状态码: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);

        // 写入临时文件，完成后重命名（避免中断导致文件损坏）
        let temp_path = path.with_extension("bin.tmp");
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;

        let bytes = response.bytes().map_err(|e| format!("读取响应失败: {}", e))?;
        file.write_all(&bytes).map_err(|e| format!("写入文件失败: {}", e))?;
        file.flush().map_err(|e| format!("刷新文件失败: {}", e))?;
        drop(file);

        // 重命名为正式文件
        std::fs::rename(&temp_path, &path)
            .map_err(|e| format!("重命名文件失败: {}", e))?;

        let downloaded = if total_size > 0 { total_size } else { bytes.len() as u64 };
        tracing::info!("模型下载完成: {} bytes", downloaded);
        Ok(downloaded)
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
