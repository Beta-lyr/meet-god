pub mod schema;

use anyhow::{Context, Result};
use schema::AppConfig;
use std::fs;
use std::path::PathBuf;

/// 获取配置文件路径: %APPDATA%/meet-god/config.yaml
fn config_path() -> PathBuf {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data).join("meet-god").join("config.yaml")
}

/// 加载配置，不存在则返回默认值
pub fn load() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_yaml::from_str::<AppConfig>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    tracing::warn!("配置文件解析失败，使用默认配置: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!("配置文件读取失败，使用默认配置: {}", e);
            }
        }
    }
    AppConfig::default()
}

/// 保存配置到文件
pub fn save(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建配置目录失败: {:?}", parent))?;
    }
    let content = serde_yaml::to_string(config)
        .context("配置序列化失败")?;
    fs::write(&path, content)
        .with_context(|| format!("写入配置文件失败: {:?}", path))?;
    Ok(())
}
