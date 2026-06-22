use super::{SttError, Transcription};
use crate::config::schema::SttApiConfig;
use std::time::Instant;

/// 云端 Whisper API STT Provider
///
/// 支持 OpenAI Whisper API 及兼容接口。
pub struct WhisperApiProvider {
    api_key: String,
    api_url: String,
    model: String,
    timeout_ms: u64,
}

impl WhisperApiProvider {
    pub fn new(config: &SttApiConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_url: config.api_url.clone(),
            model: config.model.clone(),
            timeout_ms: config.timeout_ms,
        }
    }
}

impl super::provider::SttProvider for WhisperApiProvider {
    fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<Transcription, SttError> {
        let start = Instant::now();

        if self.api_key.is_empty() {
            return Err(SttError::ConfigError(
                "API Key 未配置".to_string(),
            ));
        }

        // f32 PCM 转 WAV 字节
        let wav_bytes = pcm_to_wav(audio, sample_rate);

        let duration_ms = (audio.len() as f64 / sample_rate as f64 * 1000.0) as u64;

        // 使用 reqwest 发送 multipart 请求
        let url = format!("{}/audio/transcriptions", self.api_url);
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let timeout = self.timeout_ms;

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout))
            .build()
            .map_err(|e| SttError::NetworkError(e.to_string()))?;

        let form = reqwest::blocking::multipart::Form::new()
            .part(
                "file",
                reqwest::blocking::multipart::Part::bytes(wav_bytes)
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| SttError::NetworkError(e.to_string()))?,
            )
            .text("model", model)
            .text("language", "zh");

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .map_err(|e| SttError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(SttError::NetworkError(format!(
                "API 返回错误 {}: {}",
                status, body
            )));
        }

        let result: serde_json::Value = response
            .json()
            .map_err(|e| SttError::NetworkError(e.to_string()))?;

        let text = result["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(Transcription {
            text,
            confidence: 1.0, // API 不返回置信度
            language: "zh".to_string(),
            duration_ms,
            latency_ms,
        })
    }

    fn name(&self) -> &str {
        "whisper-api"
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// f32 PCM 数据转 WAV 格式字节
fn pcm_to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // samples: f32 -> i16
    for &sample in samples {
        let clamped = sample.max(-1.0).min(1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&i16_sample.to_le_bytes());
    }

    wav
}
