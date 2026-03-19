/// Shared IPC state between the Zalo WebView and the HTTP route handlers.
///
/// The Zalo sidebar WebView posts data via window.ipc.postMessage(). The event
/// loop stores it here. HTTP handlers read/clear the buffers to serve responses.
use std::sync::Mutex;

/// Buffer for the raw message list sent by Zalo WebView JS.
pub static ZALO_MESSAGES: Mutex<Option<serde_json::Value>> = Mutex::new(None);

/// Buffer for the conversation list sent by Zalo WebView JS.
pub static ZALO_CONVERSATIONS: Mutex<Option<serde_json::Value>> = Mutex::new(None);

/// Pending JS snippets to evaluate on the Zalo sidebar WebView.
/// Each entry is (js_code, result_sender). The event loop drains this queue
/// on every ProcessJsQueue tick and sends "ok" / "error:..." back.
pub static ZALO_JS_QUEUE: Mutex<Vec<(String, std::sync::mpsc::Sender<String>)>> =
    Mutex::new(Vec::new());

/// Enqueue a JS snippet for execution on the Zalo WebView and block until the
/// event loop processes it (up to 10 s timeout). Returns the result string.
pub fn eval_zalo_js(js: &str) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut queue = ZALO_JS_QUEUE.lock().unwrap();
        queue.push((js.to_string(), tx));
    }
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| format!("JS eval timeout: {}", e))
}

/// Event variants for the tao UserEvent type parameter.
#[derive(Clone)]
pub enum UserEvent {
    /// Signals the event loop to drain ZALO_JS_QUEUE.
    ProcessJsQueue,
}
