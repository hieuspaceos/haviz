/// Zalo WebView message accumulator.
///
/// Drains the ZALO_MESSAGES IPC buffer (populated by JS_EXTRACT_MESSAGES),
/// parses raw text entries into ParsedMessage, and upserts them into SQLite.
/// Called periodically by the background polling task in haviz_app.rs.
use std::sync::Arc;

use crate::app::ipc::{eval_zalo_js, ZALO_MESSAGES};
use crate::db::Database;
use crate::message_parser::{compute_hash, ParsedMessage};
use crate::routes::zalo_scripts::JS_EXTRACT_MESSAGES;

/// Parse the raw IPC JSON value (array of {sender, content, class} objects)
/// produced by JS_EXTRACT_MESSAGES into ParsedMessage list.
/// JS does not know sender name — all extracted texts are treated as "inbound"
/// unless the content hash matches an outbound pattern set by caller.
fn parse_ipc_messages(raw: &serde_json::Value) -> Vec<ParsedMessage> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => {
            // May be wrapped: { type: ..., data: [...] } or bare array
            if let Some(data) = raw.get("data").and_then(|d| d.as_array()) {
                return parse_array(data);
            }
            return vec![];
        }
    };
    parse_array(arr)
}

/// System/UI messages that should never be stored in DB.
const SKIP_MESSAGES: &[&str] = &[
    "Kích hoạt", "Activate", "Tải ngay", "Đồng bộ ngay",
    "Nhấn kích hoạt để sử dụng trên Tab này",
    "Bạn đang mở Zalo trên một Tab khác hoặc không sử dụng Zalo quá lâu",
    "Đồng bộ tin nhắn gần đây", "Nhấn để đồng bộ ngay",
    "Zalo Web của bạn hiện chưa có đầy đủ tin nhắn gần đây",
    "Chưa có tin nhắn", "Đang kết nối", "Đang tải",
    "Tin nhắn", "Danh bạ", "Zalo Cloud", "My Documents",
    "Công cụ", "Cài đặt", "Tìm kiếm", "Tất cả", "Chưa đọc", "Phân loại",
    "Gửi nhanh", "Gọi lại",
];

fn parse_array(arr: &[serde_json::Value]) -> Vec<ParsedMessage> {
    let mut out = Vec::new();
    for item in arr {
        let content = match item.get("content").and_then(|c| c.as_str()) {
            Some(c) if c.len() >= 2 => c.to_string(),
            _ => continue,
        };
        // Skip system/UI messages
        if SKIP_MESSAGES.iter().any(|&s| content == s) {
            continue;
        }
        // Sender from JS is usually empty ("") — use empty string, direction = inbound
        let sender = item
            .get("sender")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let hash = compute_hash(&sender, &content, &timestamp);
        out.push(ParsedMessage {
            sender,
            content,
            timestamp,
            direction: "inbound".to_string(),
            content_hash: hash,
        });
    }
    out
}

/// Trigger JS extraction and wait up to 2s for the IPC buffer to fill.
/// Returns None if Zalo WebView is not ready / JS didn't fire.
fn trigger_and_collect() -> Option<serde_json::Value> {
    // Clear old buffer first
    *ZALO_MESSAGES.lock().unwrap() = None;

    // Enqueue extraction — non-blocking fire (timeout 10s in eval, but we don't wait here)
    // We use a background thread to avoid blocking the accumulator loop.
    let _ = eval_zalo_js(JS_EXTRACT_MESSAGES);

    // Wait up to 2s for the IPC handler to populate the buffer
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if ZALO_MESSAGES.lock().unwrap().is_some() {
            break;
        }
    }

    ZALO_MESSAGES.lock().unwrap().take()
}

/// One accumulation cycle: extract from WebView → parse → upsert new msgs to DB.
/// Returns count of new messages stored, or error string.
pub fn accumulate_once(db: &Arc<Database>) -> Result<usize, String> {
    let raw = match trigger_and_collect() {
        Some(v) => v,
        None => return Err("zalo_not_ready".to_string()),
    };

    let messages = parse_ipc_messages(&raw);
    if messages.is_empty() {
        return Ok(0);
    }

    let mut new_count = 0;
    for msg in messages {
        // Dedup check
        match db.message_exists_by_hash(&msg.content_hash) {
            Ok(true) => continue,
            Ok(false) => {}
            Err(e) => {
                tracing::warn!("DB dedup check failed: {}", e);
                continue;
            }
        }

        let preview = if msg.content.chars().count() > 50 {
            format!("{}...", msg.content.chars().take(50).collect::<String>())
        } else {
            msg.content.clone()
        };

        let conv_id = match db.upsert_conversation(&msg.sender, &preview, &msg.direction) {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!("upsert_conversation failed: {}", e);
                continue;
            }
        };

        match db.insert_message(
            &conv_id,
            &msg.direction,
            &msg.sender,
            &msg.content,
            &msg.content_hash,
            &msg.timestamp,
        ) {
            Ok(_) => {
                new_count += 1;
            }
            Err(e) => {
                tracing::warn!("insert_message failed: {}", e);
            }
        }
    }

    Ok(new_count)
}
