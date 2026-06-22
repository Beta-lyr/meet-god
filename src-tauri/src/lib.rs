mod audio;
mod config;
mod llm;
mod pipeline;
mod stealth;
mod stt;

use audio::capture::AudioCaptureHandle;
use config::schema::AppConfig;
use pipeline::engine::{PipelineEngine, PipelineEvent};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::Emitter;
use tokio::sync::Mutex;

/// 应用状态，通过 Tauri 管理
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub audio_handle: Mutex<Option<AudioCaptureHandle>>,
    pub pipeline: Mutex<Option<Arc<PipelineEngine>>>,
    pub running: Arc<Mutex<bool>>,
}

/// 音频设备信息
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// 系统信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub arch: String,
}

/// 管线状态
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub running: bool,
    pub audio_state: String,
    pub stt_provider: String,
    pub llm_provider: String,
}

// ========== Tauri Commands ==========

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    state: tauri::State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), String> {
    config::save(&new_config).map_err(|e| e.to_string())?;
    let mut config = state.config.lock().await;
    *config = new_config;
    Ok(())
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        os_version: std::env::var("OS").unwrap_or_else(|_| "unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[tauri::command]
fn get_audio_device_info() -> Result<AudioDeviceInfo, String> {
    let name = audio::capture::get_default_output_device_name().map_err(|e| e.to_string())?;
    let sample_rate =
        audio::capture::get_default_output_sample_rate().map_err(|e| e.to_string())?;
    Ok(AudioDeviceInfo {
        name,
        sample_rate,
        channels: 2,
    })
}

#[tauri::command]
fn set_invisible_to_capture(window: tauri::Window) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_invisible_to_capture(hwnd.0 as isize)?;
    }
    Ok(())
}

#[tauri::command]
fn set_always_on_top(window: tauri::Window, on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(on_top)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_window_opacity(window: tauri::Window, opacity: f64) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_opacity(hwnd.0 as isize, opacity)?;
    }
    Ok(())
}

#[tauri::command]
fn get_whisper_model_status(model_name: String) -> serde_json::Value {
    let provider = stt::whisper_local::WhisperLocalProvider::new(
        &config::schema::WhisperLocalConfig {
            model: model_name.clone(),
            ..Default::default()
        },
    );
    serde_json::json!({
        "model": model_name,
        "exists": provider.model_exists(),
        "path": provider.get_model_path().to_string_lossy(),
        "download_url": stt::whisper_local::WhisperLocalProvider::model_download_url(&model_name),
    })
}

/// 启动管线（音频捕获 + STT + LLM + 事件推送）
#[tauri::command]
async fn start_pipeline(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 检查是否已运行
    {
        let running = state.running.lock().await;
        if *running {
            return Err("管线已在运行中".to_string());
        }
    }

    let config = state.config.lock().await;

    // 创建 STT Provider
    let stt_provider: Arc<dyn stt::provider::SttProvider> =
        Arc::from(stt::provider::create_provider(&config.stt));
    if !stt_provider.is_ready() {
        return Err(format!("STT 未就绪: {} (可能需要下载模型)", stt_provider.name()));
    }

    // 创建 LLM Provider
    let llm_provider: Arc<dyn llm::provider::LlmProvider> =
        Arc::from(llm::provider::create_provider(&config.llm.primary));
    if !llm_provider.is_ready() {
        return Err(format!("LLM 未就绪: {} (请配置 API Key)", llm_provider.name()));
    }

    let system_prompt = build_system_prompt(&config.profile);
    let chat_config = llm::ChatConfig {
        model: config.llm.primary.model.clone(),
        temperature: config.llm.primary.temperature,
        max_tokens: config.llm.primary.max_tokens,
        system_prompt,
    };

    let vad_threshold = config.audio.vad_threshold;
    let sample_rate = config.audio.sample_rate;
    drop(config);

    // 创建管线引擎
    let engine = Arc::new(PipelineEngine::new(
        stt_provider,
        llm_provider,
        chat_config,
        vad_threshold,
    ));

    // 启动音频捕获
    let buffer = Arc::new(StdMutex::new(Vec::<f32>::new()));
    let capture_state = Arc::new(StdMutex::new(audio::CaptureState::Running));

    let audio_handle = audio::capture::start_capture(sample_rate, buffer.clone(), capture_state)
        .map_err(|e| format!("音频捕获启动失败: {}", e))?;

    // 存入状态
    *state.audio_handle.lock().await = Some(audio_handle);
    *state.pipeline.lock().await = Some(engine.clone());
    *state.running.lock().await = true;

    // 启动事件推送循环（后台任务）
    let engine_clone = engine.clone();
    let app_handle_clone = app_handle.clone();
    let running_flag = state.running.clone(); // Arc<Mutex<bool>> - Arc is Clone

    tokio::spawn(async move {
        tracing::info!("事件推送循环已启动");

        loop {
            // 检查是否应该继续运行
            {
                let running = running_flag.lock().await;
                if !*running {
                    break;
                }
            }

            // 从缓冲区取出音频数据（MutexGuard 必须在 await 前 drop）
            let audio_data = {
                let mut buf = buffer.lock().unwrap();
                let data = if buf.len() >= 16000 {
                    Some(std::mem::take(&mut *buf))
                } else {
                    None
                };
                drop(buf); // 显式 drop，确保在 await 前释放
                data
            };

            let audio_data = match audio_data {
                Some(data) => data,
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    continue;
                }
            };

            // 处理音频
            if let Some(event) = engine_clone.process_audio(&audio_data).await {
                // 推送 STT 事件到前端
                let _ = app_handle_clone.emit("pipeline-event", &event);

                // 如果是识别结果，调用 LLM 生成答案
                if let PipelineEvent::Transcription { ref text, .. } = event {
                    let answer_event = engine_clone.generate_answer(text).await;
                    let _ = app_handle_clone.emit("pipeline-event", &answer_event);
                }
            }
        }

        tracing::info!("事件推送循环已停止");
    });

    // 推送状态变化事件
    let _ = app_handle.emit("pipeline-event", PipelineEvent::StateChange {
        state: "running".to_string(),
    });

    tracing::info!("管线已启动");
    Ok(())
}

/// 停止管线
#[tauri::command]
async fn stop_pipeline(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    *state.running.lock().await = false;
    *state.audio_handle.lock().await = None;
    *state.pipeline.lock().await = None;

    let _ = app_handle.emit("pipeline-event", PipelineEvent::StateChange {
        state: "stopped".to_string(),
    });

    tracing::info!("管线已停止");
    Ok(())
}

/// 切换静音
#[tauri::command]
async fn toggle_mute(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let handle = state.audio_handle.lock().await;
    if let Some(ref h) = *handle {
        let new_state = h.toggle_mute();
        Ok(format!("{:?}", new_state))
    } else {
        Err("音频捕获未启动".to_string())
    }
}

/// 获取管线状态
#[tauri::command]
async fn get_pipeline_status(state: tauri::State<'_, AppState>) -> Result<PipelineStatus, String> {
    let handle = state.audio_handle.lock().await;
    let pipeline = state.pipeline.lock().await;
    let config = state.config.lock().await;

    Ok(PipelineStatus {
        running: pipeline.is_some(),
        audio_state: match handle.as_ref().map(|h| h.state()) {
            Some(audio::CaptureState::Running) => "running".to_string(),
            Some(audio::CaptureState::Muted) => "muted".to_string(),
            Some(audio::CaptureState::Idle) => "idle".to_string(),
            Some(audio::CaptureState::Error) => "error".to_string(),
            None => "stopped".to_string(),
        },
        stt_provider: config.stt.provider.clone(),
        llm_provider: config.llm.primary.provider.clone(),
    })
}

/// 手动发送文本到 LLM（测试用，不经过 STT）
#[tauri::command]
async fn process_audio_text(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<String, String> {
    let pipeline = state.pipeline.lock().await;
    let config = state.config.lock().await;

    if let Some(ref engine) = *pipeline {
        let event = engine.generate_answer(&text).await;
        match event {
            PipelineEvent::AnswerChunk { content, .. } => Ok(content),
            PipelineEvent::Error { message } => Err(message),
            _ => Ok(String::new()),
        }
    } else {
        // 管线未启动时，直接调用 LLM
        let llm_provider: Arc<dyn llm::provider::LlmProvider> =
            Arc::from(llm::provider::create_provider(&config.llm.primary));
        let system_prompt = build_system_prompt(&config.profile);

        let messages = vec![
            llm::Message {
                role: llm::Role::System,
                content: system_prompt,
            },
            llm::Message {
                role: llm::Role::User,
                content: text,
            },
        ];

        let chat_config = llm::ChatConfig {
            model: config.llm.primary.model.clone(),
            temperature: config.llm.primary.temperature,
            max_tokens: config.llm.primary.max_tokens,
            system_prompt: String::new(),
        };

        llm_provider
            .chat(&messages, &chat_config)
            .map_err(|e| e.to_string())
    }
}

/// 构建 System Prompt
fn build_system_prompt(profile: &config::schema::ProfileConfig) -> String {
    if !profile.custom_prompt.is_empty() {
        return profile.custom_prompt.clone();
    }

    let mut prompt = String::from(
        "你是一位资深面试官助理。当求职者在面试中被提问时，你需要：\n\
         1. 快速理解面试官的问题意图\n\
         2. 生成结构清晰、要点明确的参考答案\n\
         3. 如果是技术问题，提供代码示例\n\
         4. 如果是行为面试问题，使用 STAR 法则组织答案\n\
         5. 答案控制在 200 字以内，便于快速阅读",
    );

    if !profile.resume.is_empty() {
        prompt.push_str(&format!("\n\n当前简历信息：{}", profile.resume));
    }
    if !profile.job_description.is_empty() {
        prompt.push_str(&format!("\n\n目标岗位描述：{}", profile.job_description));
    }
    prompt
}

// ========== Tauri 应用入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let app_config = config::load();
    tracing::info!("配置已加载");

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(app_config),
            audio_handle: Mutex::new(None),
            pipeline: Mutex::new(None),
            running: Arc::new(Mutex::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_system_info,
            get_audio_device_info,
            set_invisible_to_capture,
            set_always_on_top,
            set_window_opacity,
            get_whisper_model_status,
            start_pipeline,
            stop_pipeline,
            toggle_mute,
            get_pipeline_status,
            process_audio_text,
        ])
        .setup(|_app| {
            // 窗口初始化由前端挂载后通过 set_invisible_to_capture 命令完成
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
