/// Route handlers for Zalo Web sidebar control.
///
/// All handlers interact with the Zalo WebView via eval_zalo_js() (IPC queue) or
/// via AppleScript run_osascript() for OS-level mouse/keyboard actions.
/// JS snippets live in zalo_scripts.rs to keep this file under 200 LOC.
use axum::{extract::Query, response::Json, response::Response};
use serde::Deserialize;

use crate::app::ipc::{eval_zalo_js, ZALO_CONVERSATIONS, ZALO_MESSAGES};
#[cfg(target_os = "macos")]
use crate::platform::macos::osascript::run_osascript;

use super::zalo_scripts as js;

// ── Request structs ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchRequest { pub query: String }

#[derive(Deserialize)]
pub struct OpenRequest { pub index: usize }

#[derive(Deserialize)]
pub struct SendMsgRequest { pub message: String }

#[derive(Deserialize)]
pub struct SearchAndSendRequest { pub to: String, pub message: String }

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Wait up to 2s for an IPC result to arrive.
fn wait_for_ipc<T: Clone>(mutex: &std::sync::Mutex<Option<T>>) -> Option<T> {
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Some(val) = mutex.lock().unwrap().as_ref() {
            return Some(val.clone());
        }
    }
    mutex.lock().unwrap().take()
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/zalo/conversations — extract conversation list from sidebar.
pub async fn zalo_conversations_handler() -> Json<serde_json::Value> {
    *ZALO_CONVERSATIONS.lock().unwrap() = None;
    let _ = eval_zalo_js(js::JS_EXTRACT_CONVERSATIONS);
    let data = wait_for_ipc(&ZALO_CONVERSATIONS);
    match data {
        Some(convs) => Json(serde_json::json!({ "ok": true, "conversations": convs })),
        None => Json(serde_json::json!({ "ok": true, "conversations": [] })),
    }
}

/// GET /api/zalo/messages — scroll up to load history, then extract messages.
pub async fn zalo_messages_handler() -> Json<serde_json::Value> {
    // Scroll up to trigger Zalo lazy-loading of older messages
    let _ = eval_zalo_js(js::JS_SCROLL_UP_CHAT);
    // Wait for scrolls to complete and Zalo to render older messages
    std::thread::sleep(std::time::Duration::from_millis(2500));

    *ZALO_MESSAGES.lock().unwrap() = None;
    let _ = eval_zalo_js(js::JS_EXTRACT_MESSAGES);
    let data = wait_for_ipc(&ZALO_MESSAGES);
    match data {
        Some(msgs) => Json(serde_json::json!({ "ok": true, "messages": msgs })),
        None => Json(serde_json::json!({
            "ok": false, "messages": [],
            "note": "No messages extracted. Open a conversation in Zalo sidebar first.",
        })),
    }
}

/// Internal GET /api/zalo/_messages_callback — receives messages via query param.
pub async fn zalo_messages_callback(
    query: Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Some(data) = query.get("data") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
            *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
        }
    }
    // Return 1×1 transparent PNG so JS image.src trick gets a valid response
    axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
            0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]))
        .unwrap()
}

/// POST /api/zalo/search — type query into search input and press Enter.
pub async fn zalo_search_handler(
    axum::extract::Json(req): axum::extract::Json<SearchRequest>,
) -> Json<serde_json::Value> {
    let _ = eval_zalo_js(js::JS_CLEAR_INPUT);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = eval_zalo_js(&js::js_type_search(&req.query));
    std::thread::sleep(std::time::Duration::from_millis(2500));
    if !req.query.is_empty() {
        let _ = eval_zalo_js(js::JS_ENTER_SEARCH);
    }
    Json(serde_json::json!({ "ok": true, "query": req.query }))
}

/// POST /api/zalo/open — click the Nth conversation item (1-based index).
pub async fn zalo_open_handler(
    axum::extract::Json(req): axum::extract::Json<OpenRequest>,
) -> Json<serde_json::Value> {
    let _ = eval_zalo_js(&js::js_click_conversation(req.index.saturating_sub(1)));
    Json(serde_json::json!({ "ok": true, "clicked_index": req.index }))
}

/// POST /api/zalo/send — type message and send in current conversation.
pub async fn zalo_send_handler(
    axum::extract::Json(req): axum::extract::Json<SendMsgRequest>,
) -> Json<serde_json::Value> {
    let message = req.message.clone();
    tokio::task::spawn_blocking(move || {
        // Focus chat input — platform-specific
        #[cfg(target_os = "macos")]
        focus_chat_input_macos();
        #[cfg(target_os = "windows")]
        { eval_zalo_js(js::JS_FOCUS_CHAT_INPUT).ok(); }

        std::thread::sleep(std::time::Duration::from_millis(500));
        eval_zalo_js(&js::js_type_message(&message)).ok();
        std::thread::sleep(std::time::Duration::from_millis(500));
        eval_zalo_js(js::JS_SEND_ENTER).ok();
    }).await.ok();
    Json(serde_json::json!({ "ok": true, "message": req.message }))
}

/// POST /api/zalo/search-and-send — full flow: search → open → type → send.
pub async fn zalo_search_and_send_handler(
    axum::extract::Json(req): axum::extract::Json<SearchAndSendRequest>,
) -> Json<serde_json::Value> {
    let to = req.to.clone();
    let message = req.message.clone();
    let result = tokio::task::spawn_blocking(move || {
        eval_zalo_js(&js::js_clear_and_type_search(&to)).ok();
        std::thread::sleep(std::time::Duration::from_millis(2500));
        eval_zalo_js(js::JS_ENTER_SEARCH).ok();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        eval_zalo_js(&js::js_type_message(&message)).ok();
        std::thread::sleep(std::time::Duration::from_millis(500));
        eval_zalo_js(js::JS_SEND_ENTER).ok();
        Ok::<_, String>("sent".to_string())
    }).await.unwrap_or(Err("spawn failed".to_string()));
    Json(serde_json::json!({ "ok": result.is_ok(), "to": req.to, "message": req.message }))
}

// ── Platform helpers ─────────────────────────────────────────────────────────

/// macOS: use AppleScript to OS-level click into chat input area.
#[cfg(target_os = "macos")]
fn focus_chat_input_macos() {
    let _ = run_osascript("tell application \"System Events\" to click at {1, 1}");
    let pos = run_osascript(
        "tell application \"System Events\" to tell process \"haviz_app\" \
         to return {position of window 1, size of window 1}",
    );
    match pos {
        Ok(s) => {
            let nums: Vec<f64> = s.replace('{', "").replace('}', "")
                .split(',').filter_map(|s| s.trim().parse().ok()).collect();
            if nums.len() >= 4 {
                let (wx, wy, ww, wh) = (nums[0], nums[1], nums[2], nums[3]);
                let _ = run_osascript(&format!(
                    "tell application \"System Events\" to click at {{{}, {}}}",
                    (wx + ww - 200.0) as i64, (wy + wh - 60.0) as i64
                ));
            }
        }
        Err(_) => { eval_zalo_js(js::JS_FOCUS_CHAT_INPUT).ok(); }
    }
}
