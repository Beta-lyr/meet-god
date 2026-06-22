mod audio;
mod config;
mod llm;
mod pipeline;
mod stealth;
mod stt;

use config::schema::AppConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// 应用状态，通过 Tauri 管理
pub struct AppState {
    pub config: Mutex<AppConfig>,
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

// ========== Tauri Commands ==========

/// 获取应用配置
#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

/// 保存应用配置
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

/// 获取系统信息
#[tauri::command]
fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        os_version: std::env::var("OS").unwrap_or_else(|_| "unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
    }
}

/// 获取默认音频设备信息
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

/// 设置窗口对屏幕捕获不可见
#[tauri::command]
fn set_invisible_to_capture(window: tauri::Window) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_invisible_to_capture(hwnd.0 as isize)?;
    }
    Ok(())
}

/// 设置窗口置顶
#[tauri::command]
fn set_always_on_top(window: tauri::Window, on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(on_top)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 设置窗口透明度
#[tauri::command]
fn set_window_opacity(window: tauri::Window, opacity: f64) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        stealth::window::set_opacity(hwnd.0 as isize, opacity)?;
    }
    Ok(())
}

/// 获取 Whisper 模型状态
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

// ========== Tauri 应用入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let app_config = config::load();
    tracing::info!("配置已加载");

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(app_config),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
