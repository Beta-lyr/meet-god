use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// 全局数据库连接（线程安全）
fn db_conn() -> &'static Mutex<Option<Connection>> {
    static DB: OnceLock<Mutex<Option<Connection>>> = OnceLock::new();
    DB.get_or_init(|| Mutex::new(None))
}

/// 获取数据库文件路径: %APPDATA%/meet-god/data/sessions.db
fn db_path() -> PathBuf {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))
        .unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(app_data).join("meet-god").join("data");
    fs::create_dir_all(&dir).ok();
    dir.join("sessions.db")
}

/// 会话记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub message_count: i64,
}

/// 会话消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub latency_ms: u64,
    pub created_at: String,
    pub bookmark: Option<String>,
}

/// 初始化数据库，创建表结构
pub fn init_database() -> Result<(), String> {
    let path = db_path();
    tracing::info!("数据库路径: {:?}", path);

    let conn = Connection::open(&path).map_err(|e| format!("打开数据库失败: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            started_at TEXT NOT NULL,
            ended_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            latency_ms INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            bookmark TEXT,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );"
    ).map_err(|e| format!("创建表失败: {}", e))?;

    let mut db = db_conn().lock().map_err(|e| format!("获取锁失败: {}", e))?;
    *db = Some(conn);

    tracing::info!("数据库初始化完成");
    Ok(())
}

/// 获取数据库连接的闭包辅助函数
fn with_db<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Connection) -> Result<R, String>,
{
    let guard = db_conn().lock().map_err(|e| format!("获取锁失败: {}", e))?;
    let conn = guard.as_ref().ok_or("数据库未初始化")?;
    f(conn)
}

/// 创建新会话
pub fn create_session(id: &str, title: &str) -> Result<Session, String> {
    let now = chrono_now();
    with_db(|conn| {
        conn.execute(
            "INSERT INTO sessions (id, title, started_at, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, now, now],
        ).map_err(|e| format!("创建会话失败: {}", e))?;

        Ok(Session {
            id: id.to_string(),
            title: title.to_string(),
            started_at: now.clone(),
            ended_at: None,
            message_count: 0,
        })
    })
}

/// 结束会话
pub fn end_session(id: &str) -> Result<(), String> {
    let now = chrono_now();
    with_db(|conn| {
        let rows = conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            params![now, id],
        ).map_err(|e| format!("结束会话失败: {}", e))?;

        if rows == 0 {
            Err(format!("会话不存在: {}", id))
        } else {
            Ok(())
        }
    })
}

/// 添加消息
pub fn add_message(
    session_id: &str,
    role: &str,
    content: &str,
    latency_ms: u64,
) -> Result<Message, String> {
    let id = uuid();
    let now = chrono_now();
    with_db(|conn| {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, latency_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, session_id, role, content, latency_ms as i64, now],
        ).map_err(|e| format!("添加消息失败: {}", e))?;

        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            latency_ms,
            created_at: now,
            bookmark: None,
        })
    })
}

/// 更新书签
pub fn update_bookmark(message_id: &str, bookmark: Option<&str>) -> Result<(), String> {
    with_db(|conn| {
        let rows = conn.execute(
            "UPDATE messages SET bookmark = ?1 WHERE id = ?2",
            params![bookmark, message_id],
        ).map_err(|e| format!("更新书签失败: {}", e))?;

        if rows == 0 {
            Err(format!("消息不存在: {}", message_id))
        } else {
            Ok(())
        }
    })
}

/// 列出会话（按创建时间倒序）
pub fn list_sessions(limit: i64) -> Result<Vec<Session>, String> {
    with_db(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.title, s.started_at, s.ended_at, COUNT(m.id) as message_count
                 FROM sessions s
                 LEFT JOIN messages m ON m.session_id = s.id
                 GROUP BY s.id
                 ORDER BY s.created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let sessions = stmt
            .query_map(params![limit], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    message_count: row.get(4)?,
                })
            })
            .map_err(|e| format!("查询会话列表失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sessions)
    })
}

/// 获取会话的所有消息
pub fn get_session_messages(session_id: &str) -> Result<Vec<Message>, String> {
    with_db(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, role, content, latency_ms, created_at, bookmark
                 FROM messages
                 WHERE session_id = ?1
                 ORDER BY created_at ASC",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let messages = stmt
            .query_map(params![session_id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    latency_ms: row.get::<_, i64>(4)? as u64,
                    created_at: row.get(5)?,
                    bookmark: row.get(6)?,
                })
            })
            .map_err(|e| format!("查询消息失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    })
}

/// 删除会话及其消息
pub fn delete_session(id: &str) -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![id])
            .map_err(|e| format!("删除消息失败: {}", e))?;

        let rows = conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])
            .map_err(|e| format!("删除会话失败: {}", e))?;

        if rows == 0 {
            Err(format!("会话不存在: {}", id))
        } else {
            Ok(())
        }
    })
}

/// 导出会话为 Markdown 格式
pub fn export_session_markdown(session_id: &str) -> Result<String, String> {
    let sessions = list_sessions(9999)?;
    let session = sessions
        .iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("会话不存在: {}", session_id))?;

    let messages = get_session_messages(session_id)?;

    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", if session.title.is_empty() { "会话记录" } else { &session.title }));
    md.push_str(&format!("- 开始时间: {}\n", session.started_at));
    if let Some(ref ended) = session.ended_at {
        md.push_str(&format!("- 结束时间: {}\n", ended));
    }
    md.push_str(&format!("- 消息数: {}\n\n", session.message_count));
    md.push_str("---\n\n");

    for msg in &messages {
        let role_label = if msg.role == "question" { "问题" } else { "回答" };
        md.push_str(&format!("### {}\n\n", role_label));
        md.push_str(&format!("{}\n\n", msg.content));
        if msg.latency_ms > 0 {
            md.push_str(&format!("`延迟: {}ms`\n\n", msg.latency_ms));
        }
        if let Some(ref bm) = msg.bookmark {
            md.push_str(&format!("`标签: {}`\n\n", bm));
        }
        md.push_str("---\n\n");
    }

    Ok(md)
}

/// 导出会话为 JSON 格式
pub fn export_session_json(session_id: &str) -> Result<String, String> {
    let sessions = list_sessions(9999)?;
    let session = sessions
        .iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("会话不存在: {}", session_id))?;

    let messages = get_session_messages(session_id)?;

    let export = serde_json::json!({
        "session": session,
        "messages": messages,
    });

    serde_json::to_string_pretty(&export).map_err(|e| format!("JSON 序列化失败: {}", e))
}

// ========== 工具函数 ==========

/// 生成 UUID v4
fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}-{:04x}-{:04x}", t, rand_u16(), rand_u16())
}

/// 获取当前时间 ISO 8601 格式
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // 简单的 UTC 时间格式化
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // 从 1970-01-01 开始计算年月日
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    days += 719468;
    let era = days / 146097;
    let doe = days % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn rand_u16() -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    hasher.finish() as u16
}
