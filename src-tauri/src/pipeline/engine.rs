use crate::audio::vad::VadDetector;
use crate::stt::provider::SttProvider;
use crate::llm::provider::LlmProvider;
use crate::llm::{ChatConfig, Message, Role};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// 管线事件，发送到前端
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineEvent {
    /// STT 识别结果
    Transcription {
        text: String,
        confidence: f32,
        latency_ms: u64,
    },
    /// LLM 答案（流式片段）
    AnswerChunk {
        content: String,
        done: bool,
    },
    /// 管线状态变化
    StateChange {
        state: String,
    },
    /// 错误
    Error {
        message: String,
    },
}

/// 数据管线引擎
///
/// 串联 音频捕获 → VAD → STT → LLM → 前端 的完整数据流
pub struct PipelineEngine {
    /// VAD 检测器
    vad: VadDetector,
    /// STT Provider
    stt: Arc<dyn SttProvider>,
    /// LLM Provider
    llm: Arc<dyn LlmProvider>,
    /// LLM 配置
    chat_config: ChatConfig,
    /// 音频累积缓冲区（用于语音片段拼接）
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    /// 上次识别的时间戳
    last_recognize_time: Arc<Mutex<Instant>>,
    /// 语音片段最小长度 (采样数, 16kHz 下 1s = 16000)
    min_speech_samples: usize,
    /// 语音片段最大长度 (采样数, 16kHz 下 10s = 160000)
    max_speech_samples: usize,
}

impl PipelineEngine {
    pub fn new(
        stt: Arc<dyn SttProvider>,
        llm: Arc<dyn LlmProvider>,
        chat_config: ChatConfig,
        vad_threshold: f32,
    ) -> Self {
        Self {
            vad: VadDetector::new(vad_threshold, 16000),
            stt,
            llm,
            chat_config,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            last_recognize_time: Arc::new(Mutex::new(Instant::now())),
            min_speech_samples: 16000,    // 1 秒
            max_speech_samples: 160000,   // 10 秒
        }
    }

    /// 处理新的音频数据
    ///
    /// 1. 将音频追加到缓冲区
    /// 2. 检查是否有足够长的语音片段
    /// 3. 如果有，进行 STT 识别
    /// 4. 将识别结果送入 LLM 生成答案
    pub async fn process_audio(&self, samples: &[f32]) -> Option<PipelineEvent> {
        // 追加到缓冲区
        {
            let mut buf = self.audio_buffer.lock().await;
            buf.extend_from_slice(samples);
        }

        // 检查缓冲区大小
        let buf_len = {
            let buf = self.audio_buffer.lock().await;
            buf.len()
        };

        // 缓冲区够大，进行 VAD 检测
        if buf_len >= self.min_speech_samples {
            let audio = {
                let mut buf = self.audio_buffer.lock().await;
                // 取出所有缓冲数据
                std::mem::take(&mut *buf)
            };

            // VAD 检测
            if self.vad.has_speech(&audio, 0.3) {
                tracing::info!("检测到语音，长度: {:.1}s", audio.len() as f32 / 16000.0);

                // STT 识别
                match self.stt.transcribe(&audio, 16000) {
                    Ok(transcription) => {
                        if transcription.text.trim().is_empty() {
                            tracing::debug!("识别结果为空，跳过");
                            return None;
                        }

                        tracing::info!("识别结果: {}", transcription.text);

                        // 返回 STT 事件
                        return Some(PipelineEvent::Transcription {
                            text: transcription.text.clone(),
                            confidence: transcription.confidence,
                            latency_ms: transcription.latency_ms,
                        });
                    }
                    Err(e) => {
                        tracing::error!("STT 识别失败: {}", e);
                        return Some(PipelineEvent::Error {
                            message: format!("STT 错误: {}", e),
                        });
                    }
                }
            } else {
                tracing::debug!("未检测到语音，丢弃缓冲区");
            }
        }

        None
    }

    /// 使用 LLM 生成答案
    pub async fn generate_answer(&self, question: &str) -> PipelineEvent {
        let messages = vec![
            Message {
                role: Role::System,
                content: self.chat_config.system_prompt.clone(),
            },
            Message {
                role: Role::User,
                content: question.to_string(),
            },
        ];

        match self.llm.chat_stream(&messages, &self.chat_config) {
            Ok(chunks) => {
                let full_text: String = chunks.iter().map(|c| c.content.as_str()).collect();
                if full_text.is_empty() {
                    PipelineEvent::AnswerChunk {
                        content: "(无响应)".to_string(),
                        done: true,
                    }
                } else {
                    // 返回合并的答案（完整实现应逐 chunk 推送）
                    PipelineEvent::AnswerChunk {
                        content: full_text,
                        done: true,
                    }
                }
            }
            Err(e) => {
                tracing::error!("LLM 调用失败: {}", e);
                PipelineEvent::Error {
                    message: format!("LLM 错误: {}", e),
                }
            }
        }
    }

    /// 获取音频缓冲区当前长度 (采样数)
    pub async fn buffer_len(&self) -> usize {
        self.audio_buffer.lock().await.len()
    }

    /// 清空音频缓冲区
    pub async fn clear_buffer(&self) {
        self.audio_buffer.lock().await.clear();
    }
}
