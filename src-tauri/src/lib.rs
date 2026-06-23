mod audio;
mod config;
mod llm;
mod pipeline;
mod recorder;
mod stealth;
mod stt;

use audio::capture::AudioCaptureHandle;
use config::schema::AppConfig;
use pipeline::engine::{PipelineEngine, PipelineEvent};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;

/// 应用状态，通过 Tauri 管理
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub audio_handle: Mutex<Option<AudioCaptureHandle>>,
    pub mic_handle: Mutex<Option<AudioCaptureHandle>>,
    pub pipeline: Mutex<Option<Arc<PipelineEngine>>>,
    pub running: Arc<Mutex<bool>>,
    pub current_session: Mutex<Option<String>>,
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

/// 下载 Whisper 模型文件（异步，不阻塞 UI）
#[tauri::command]
async fn download_whisper_model(model_name: String) -> Result<String, String> {
    let model_name_clone = model_name.clone();
    // 在阻塞线程中运行下载（reqwest::blocking 不能在 async 中直接用）
    let result = tokio::task::spawn_blocking(move || {
        stt::whisper_local::WhisperLocalProvider::download_model(&model_name_clone)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?;

    result.map(|bytes| format!("下载完成: {:.1} MB", bytes as f64 / 1024.0 / 1024.0))
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
        system_prompt: system_prompt.clone(),
    };

    // 创建备选引擎（如果配置了 fallback）
    let fallback_engine = if !config.llm.fallback.api_key.is_empty() {
        let fallback_llm: Arc<dyn llm::provider::LlmProvider> =
            Arc::from(llm::provider::create_provider(&config.llm.fallback));
        let fallback_config = llm::ChatConfig {
            model: config.llm.fallback.model.clone(),
            temperature: config.llm.fallback.temperature,
            max_tokens: config.llm.fallback.max_tokens,
            system_prompt,
        };
        Some(Arc::new(PipelineEngine::new(
            stt_provider.clone(),
            fallback_llm,
            fallback_config,
            config.audio.vad_threshold,
        )))
    } else {
        None
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

    // 自动创建会话
    let session_id = format!("s-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    if let Err(e) = recorder::db::create_session(&session_id, "") {
        tracing::warn!("自动创建会话失败: {}", e);
    }
    *state.current_session.lock().await = Some(session_id.clone());
    tracing::info!("自动创建会话: {}", session_id);

    // 启动事件推送循环（后台任务）
    let engine_clone = engine.clone();
    let fallback_engine_clone = fallback_engine;
    let app_handle_clone = app_handle.clone();
    let running_flag = state.running.clone(); // Arc<Mutex<bool>> - Arc is Clone
    let session_id_clone = session_id.clone();

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
                // 保存识别结果到数据库
                if let PipelineEvent::Transcription { ref text, latency_ms, .. } = event {
                    if let Err(e) = recorder::db::add_message(&session_id_clone, "question", text, latency_ms) {
                        tracing::warn!("保存问题消息失败: {}", e);
                    }
                }

                // 推送 STT 事件到前端
                let _ = app_handle_clone.emit("pipeline-event", &event);

                // 如果是识别结果，调用 LLM 生成答案（流式推送）
                if let PipelineEvent::Transcription { ref text, .. } = event {
                    let answer_events = engine_clone.generate_answer(text).await;

                    // 检查是否需要使用备选模型
                    let answer_events = if answer_events.iter().any(|e| matches!(e, PipelineEvent::Error { .. })) {
                        if let Some(ref fallback) = fallback_engine_clone {
                            tracing::warn!("主模型失败，使用备选模型");
                            let _ = app_handle_clone.emit("pipeline-event", PipelineEvent::StateChange {
                                state: "fallback".to_string(),
                            });
                            fallback.generate_answer(text).await
                        } else {
                            answer_events
                        }
                    } else {
                        answer_events
                    };

                    let mut full_answer = String::new();

                    for answer_event in answer_events {
                        // 收集完整答案用于数据库保存
                        if let PipelineEvent::AnswerChunk { ref content, done, .. } = answer_event {
                            full_answer.push_str(content);

                            // 最后一个 chunk 完成时保存到数据库
                            if done && !full_answer.is_empty() {
                                if let Err(e) = recorder::db::add_message(&session_id_clone, "answer", &full_answer, 0) {
                                    tracing::warn!("保存答案消息失败: {}", e);
                                }
                            }
                        }

                        // 逐 chunk 推送到前端（打字机效果）
                        let _ = app_handle_clone.emit("pipeline-event", &answer_event);
                    }
                }
            }
        }

        // 管线停止时自动结束会话
        if let Err(e) = recorder::db::end_session(&session_id_clone) {
            tracing::warn!("自动结束会话失败: {}", e);
        }
        tracing::info!("事件推送循环已停止，会话已结束: {}", session_id_clone);
    });

    // 推送状态变化事件（包含会话 ID）
    let _ = app_handle.emit("pipeline-event", PipelineEvent::StateChange {
        state: "running".to_string(),
    });
    let _ = app_handle.emit("session-started", serde_json::json!({ "session_id": session_id }));

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
    *state.mic_handle.lock().await = None;
    *state.pipeline.lock().await = None;

    // 结束当前会话
    {
        let current = state.current_session.lock().await;
        if let Some(ref session_id) = *current {
            if let Err(e) = recorder::db::end_session(session_id) {
                tracing::warn!("结束会话失败: {}", e);
            }
        }
    }
    *state.current_session.lock().await = None;

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

/// 切换麦克风录制（录制用户自己的声音）
#[tauri::command]
async fn toggle_microphone(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut mic = state.mic_handle.lock().await;

    if mic.is_some() {
        // 停止麦克风
        *mic = None;
        Ok("stopped".to_string())
    } else {
        // 启动麦克风
        let config = state.config.lock().await;
        let sample_rate = config.audio.sample_rate;
        drop(config);

        let buffer = Arc::new(StdMutex::new(Vec::<f32>::new()));
        let capture_state = Arc::new(StdMutex::new(audio::CaptureState::Running));

        let handle = audio::capture::start_microphone_capture(
            sample_rate,
            buffer,
            capture_state,
        )
        .map_err(|e| format!("麦克风启动失败: {}", e))?;

        let result = "running".to_string();
        *mic = Some(handle);
        Ok(result)
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
        let events = engine.generate_answer(&text).await;
        // 合并所有 chunk 为完整答案
        let full_text: String = events
            .iter()
            .filter_map(|e| match e {
                PipelineEvent::AnswerChunk { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        if full_text.is_empty() {
            match events.into_iter().next() {
                Some(PipelineEvent::Error { message }) => Err(message),
                _ => Ok("(无响应)".to_string()),
            }
        } else {
            Ok(full_text)
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

/// 切换窗口显示/隐藏
#[tauri::command]
async fn toggle_window_visibility(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 设置窗口不显示在任务栏
#[tauri::command]
fn set_no_taskbar_icon(window: tauri::Window) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_no_taskbar_icon(hwnd.0 as isize)?;
    }
    Ok(())
}

/// 设置窗口不抢夺焦点
#[tauri::command]
fn set_no_activate(window: tauri::Window) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_no_activate(hwnd.0 as isize)?;
    }
    Ok(())
}

/// 切换鼠标穿透模式
#[tauri::command]
fn set_click_through(window: tauri::Window, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_click_through(hwnd.0 as isize, enabled)?;
    }
    Ok(())
}

/// 注册全局快捷键（前端调用，备用方案）
#[tauri::command]
async fn register_hotkeys(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let app_handle_clone = app_handle.clone();

    // Ctrl+Shift+H -> 切换窗口可见性
    app_handle
        .global_shortcut()
        .on_shortcut(
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH),
            move |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = app_handle_clone.emit("hotkey", serde_json::json!({ "action": "toggle_visibility" }));
                }
            },
        )
        .map_err(|e| e.to_string())?;

    let app_handle_clone2 = app_handle.clone();

    // Ctrl+Shift+M -> 切换静音
    app_handle
        .global_shortcut()
        .on_shortcut(
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM),
            move |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = app_handle_clone2.emit("hotkey", serde_json::json!({ "action": "toggle_mute" }));
                }
            },
        )
        .map_err(|e| e.to_string())?;

    // Ctrl+Shift+Q -> 紧急退出
    app_handle
        .global_shortcut()
        .on_shortcut(
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ),
            move |app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    app.exit(0);
                }
            },
        )
        .map_err(|e| e.to_string())?;

    tracing::info!("全局快捷键已注册");
    Ok(())
}

// ========== 会话记录 Commands ==========

/// 创建新会话
#[tauri::command]
async fn create_session(
    state: tauri::State<'_, AppState>,
    title: String,
) -> Result<recorder::db::Session, String> {
    let id = format!("s-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    let session = recorder::db::create_session(&id, &title)?;
    let mut current = state.current_session.lock().await;
    *current = Some(id);
    Ok(session)
}

/// 结束会话
#[tauri::command]
async fn end_session(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    recorder::db::end_session(&session_id)?;
    let mut current = state.current_session.lock().await;
    if current.as_deref() == Some(&session_id) {
        *current = None;
    }
    Ok(())
}

/// 添加消息
#[tauri::command]
async fn add_message(
    session_id: String,
    role: String,
    content: String,
    latency_ms: u64,
) -> Result<recorder::db::Message, String> {
    recorder::db::add_message(&session_id, &role, &content, latency_ms)
}

/// 更新书签
#[tauri::command]
async fn update_bookmark(
    message_id: String,
    bookmark: Option<String>,
) -> Result<(), String> {
    recorder::db::update_bookmark(&message_id, bookmark.as_deref())
}

/// 列出会话
#[tauri::command]
async fn list_sessions(
    limit: Option<i64>,
) -> Result<Vec<recorder::db::Session>, String> {
    recorder::db::list_sessions(limit.unwrap_or(50))
}

/// 获取会话消息
#[tauri::command]
async fn get_session_messages(
    session_id: String,
) -> Result<Vec<recorder::db::Message>, String> {
    recorder::db::get_session_messages(&session_id)
}

/// 删除会话
#[tauri::command]
async fn delete_session(
    session_id: String,
) -> Result<(), String> {
    recorder::db::delete_session(&session_id)
}

/// 导出会话（markdown 或 json）
#[tauri::command]
async fn export_session(
    session_id: String,
    format: String,
) -> Result<String, String> {
    match format.as_str() {
        "markdown" | "md" => recorder::db::export_session_markdown(&session_id),
        "json" => recorder::db::export_session_json(&session_id),
        _ => Err(format!("不支持的导出格式: {}", format)),
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

// ========== 系统托盘 ==========

/// 创建系统托盘菜单
fn create_tray_menu(app: &tauri::AppHandle) -> tauri::menu::Menu<tauri::Wry> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};

    // 获取当前管线运行状态
    let running = {
        let state = app.state::<AppState>();
        // 使用 try_lock 避免在同步上下文中阻塞
        state.running.try_lock().map(|r| *r).unwrap_or(false)
    };

    let show_hide = if running {
        MenuItemBuilder::with_id("toggle_visibility", "隐藏主窗口")
            .build(app)
            .unwrap()
    } else {
        MenuItemBuilder::with_id("toggle_visibility", "显示主窗口")
            .build(app)
            .unwrap()
    };

    let start_stop = if running {
        MenuItemBuilder::with_id("toggle_pipeline", "停止")
            .build(app)
            .unwrap()
    } else {
        MenuItemBuilder::with_id("toggle_pipeline", "开始")
            .build(app)
            .unwrap()
    };

    let mute_item = MenuItemBuilder::with_id("toggle_mute", "静音")
        .build(app)
        .unwrap();

    let settings_item = MenuItemBuilder::with_id("open_settings", "设置")
        .build(app)
        .unwrap();

    let quit_item = MenuItemBuilder::with_id("quit", "退出")
        .build(app)
        .unwrap();

    let separator1 = PredefinedMenuItem::separator(app).unwrap();
    let separator2 = PredefinedMenuItem::separator(app).unwrap();

    MenuBuilder::new(app)
        .item(&show_hide)
        .item(&separator1)
        .item(&start_stop)
        .item(&mute_item)
        .item(&separator2)
        .item(&settings_item)
        .item(&quit_item)
        .build()
        .unwrap()
}

/// 设置系统托盘
fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::TrayIconBuilder;

    let menu = create_tray_menu(app);

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .tooltip("Meet God")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "toggle_visibility" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "toggle_pipeline" => {
                    let state = app.state::<AppState>();
                    let running = state.running.try_lock().map(|r| *r).unwrap_or(false);
                    if running {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app_handle.state::<AppState>();
                            *state.running.lock().await = false;
                            *state.audio_handle.lock().await = None;
                            *state.pipeline.lock().await = None;
                            let _ = app_handle.emit("pipeline-event", PipelineEvent::StateChange {
                                state: "stopped".to_string(),
                            });
                        });
                    } else {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = start_pipeline(
                                app_handle.state::<AppState>(),
                                app_handle.clone(),
                            ).await;
                        });
                    }
                }
                "toggle_mute" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = toggle_mute(app_handle.state::<AppState>()).await;
                    });
                }
                "open_settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = app.emit("navigate", "settings");
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    tracing::info!("系统托盘已创建");
    Ok(())
}

// ========== Tauri 应用入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let app_config = config::load();
    tracing::info!("配置已加载");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            config: Mutex::new(app_config),
            audio_handle: Mutex::new(None),
            mic_handle: Mutex::new(None),
            pipeline: Mutex::new(None),
            running: Arc::new(Mutex::new(false)),
            current_session: Mutex::new(None),
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
            download_whisper_model,
            start_pipeline,
            stop_pipeline,
            toggle_mute,
            toggle_microphone,
            get_pipeline_status,
            process_audio_text,
            toggle_window_visibility,
            set_no_taskbar_icon,
            set_no_activate,
            set_click_through,
            register_hotkeys,
            create_session,
            end_session,
            add_message,
            update_bookmark,
            list_sessions,
            get_session_messages,
            delete_session,
            export_session,
        ])
        .setup(|app| {
            // ========== 初始化数据库 ==========
            if let Err(e) = recorder::db::init_database() {
                tracing::error!("数据库初始化失败: {}", e);
            }

            // ========== 窗口隐蔽样式 ==========
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = window.hwnd() {
                        let h = hwnd.0 as isize;
                        // 设置任务栏不可见
                        let _ = stealth::window::set_no_taskbar_icon(h);
                        // 设置不抢夺焦点
                        let _ = stealth::window::set_no_activate(h);
                        // 设置对屏幕捕获不可见
                        let _ = stealth::window::set_invisible_to_capture(h);
                    }
                }
            }

            // ========== 全局快捷键 ==========
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

                let app_handle = app.handle().clone();

                // Ctrl+Shift+H -> 切换窗口可见性
                let ah1 = app_handle.clone();
                app.global_shortcut().on_shortcut(
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH),
                    move |_app, _shortcut, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            let _ = ah1.emit("hotkey", serde_json::json!({ "action": "toggle_visibility" }));
                        }
                    },
                )?;

                // Ctrl+Shift+M -> 切换静音
                let ah2 = app_handle.clone();
                app.global_shortcut().on_shortcut(
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM),
                    move |_app, _shortcut, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            let _ = ah2.emit("hotkey", serde_json::json!({ "action": "toggle_mute" }));
                        }
                    },
                )?;

                // Ctrl+Shift+Q -> 紧急退出
                app.global_shortcut().on_shortcut(
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ),
                    move |app, _shortcut, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            app.exit(0);
                        }
                    },
                )?;

                tracing::info!("全局快捷键已注册");
            }

            // ========== 系统托盘 ==========
            setup_tray(app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
