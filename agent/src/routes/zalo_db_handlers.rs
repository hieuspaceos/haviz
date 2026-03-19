/// DB-backed Zalo route handlers.
///
/// These handlers read accumulated messages from SQLite rather than
/// triggering a live WebView extraction. Separated from zalo_control.rs
/// to keep file sizes under 200 lines.
use axum::extract::{Query, State};
use axum::response::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::app::ipc::{eval_zalo_js, ZALO_MESSAGES};
use crate::db::Database;

use super::zalo_scripts as js;

/// Wait up to 2s for an IPC result to arrive (shared helper).
fn wait_for_ipc_local<T: Clone>(mutex: &std::sync::Mutex<Option<T>>) -> Option<T> {
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Some(val) = mutex.lock().unwrap().as_ref() {
            return Some(val.clone());
        }
    }
    mutex.lock().unwrap().take()
}

/// GET /api/zalo/messages — return accumulated messages from SQLite.
/// Falls back to live WebView snapshot if DB is empty.
/// Query params: limit (default 100).
pub async fn zalo_messages_db_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let limit: u32 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    match db.get_recent_messages(limit) {
        Ok(msgs) if !msgs.is_empty() => {
            Json(serde_json::json!({
                "ok": true,
                "source": "db",
                "count": msgs.len(),
                "messages": msgs,
            }))
        }
        Ok(_) => {
            // DB empty — trigger live extraction as fallback
            *ZALO_MESSAGES.lock().unwrap() = None;
            let _ = eval_zalo_js(js::JS_EXTRACT_MESSAGES);
            let data = wait_for_ipc_local(&ZALO_MESSAGES);
            match data {
                Some(live) => Json(serde_json::json!({
                    "ok": true,
                    "source": "live_snapshot",
                    "messages": live,
                })),
                None => Json(serde_json::json!({
                    "ok": false,
                    "source": "none",
                    "messages": [],
                    "note": "DB empty and no live snapshot. Open a conversation in Zalo sidebar first.",
                })),
            }
        }
        Err(e) => Json(serde_json::json!({
            "ok": false, "error": e.to_string(), "messages": [],
        })),
    }
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u32>,
    pub conversation_id: Option<String>,
}

/// GET /api/zalo/history — paginated accumulated messages from SQLite.
/// Query params: limit (default 200), conversation_id (optional filter).
pub async fn zalo_history_handler(
    State(db): State<Arc<Database>>,
    Query(q): Query<HistoryQuery>,
) -> Json<serde_json::Value> {
    let limit = q.limit.unwrap_or(200);

    let result = if let Some(conv_id) = &q.conversation_id {
        db.get_messages(conv_id, limit)
    } else {
        db.get_recent_messages(limit)
    };

    match result {
        Ok(msgs) => Json(serde_json::json!({
            "ok": true,
            "count": msgs.len(),
            "messages": msgs,
        })),
        Err(e) => Json(serde_json::json!({
            "ok": false, "error": e.to_string(), "messages": [],
        })),
    }
}
