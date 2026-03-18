/// Haviz App — Dashboard + Zalo Web Sidebar
///
/// Layout:
/// ┌──────────────────────────┬─────────────┐
/// │    Haviz Dashboard       │  Zalo Web   │
/// │    (localhost:9999)      │  Sidebar    │
/// │    Inbox, AI Drafts      │  (~400px)   │
/// └──────────────────────────┴─────────────┘

use std::sync::{Arc, Mutex};
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::{Rect, WebView, WebViewBuilder};
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;

const WINDOW_W: f64 = 1400.0;
const WINDOW_H: f64 = 900.0;
const SIDEBAR_W: f64 = 400.0;

/// Shared state: allows Agent server to execute JS on the Zalo sidebar WebView
static ZALO_JS_QUEUE: Mutex<Vec<(String, std::sync::mpsc::Sender<String>)>> = Mutex::new(Vec::new());

/// Queue a JS evaluation on the Zalo sidebar, returns result
pub fn eval_zalo_js(js: &str) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut queue = ZALO_JS_QUEUE.lock().unwrap();
        queue.push((js.to_string(), tx));
    }
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| format!("JS eval timeout: {}", e))
}

#[derive(Clone)]
enum UserEvent {
    ProcessJsQueue,
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║  Haviz — Revenue Intelligence         ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Start Agent HTTP server in background
    let _agent = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_agent());
    });

    std::thread::sleep(std::time::Duration::from_millis(500));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Haviz")
        .with_inner_size(LogicalSize::new(WINDOW_W, WINDOW_H))
        .build(&event_loop)
        .expect("Failed to create window");

    let store_id: [u8; 16] = *b"haviz_zalo_web_1";

    // Left: Dashboard
    let _dashboard = WebViewBuilder::new()
        .with_url("http://localhost:9999")
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(WINDOW_W - SIDEBAR_W, WINDOW_H).into(),
        })
        .with_devtools(true)
        .build_as_child(&window)
        .expect("Failed to create dashboard");

    // Right: Zalo Web
    let sidebar = WebViewBuilder::new()
        .with_url("https://chat.zalo.me")
        .with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
        .with_bounds(Rect {
            position: LogicalPosition::new(WINDOW_W - SIDEBAR_W, 0.0).into(),
            size: LogicalSize::new(SIDEBAR_W, WINDOW_H).into(),
        })
        .with_incognito(false)
        .with_data_store_identifier(store_id)
        .with_devtools(true)
        .build_as_child(&window)
        .expect("Failed to create Zalo sidebar");

    println!("✅ Haviz đang chạy!");
    println!("   Dashboard: http://localhost:9999");
    println!("   Zalo Web sidebar (phải)\n");

    // Poll JS queue periodically
    let proxy2 = proxy.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = proxy2.send_event(UserEvent::ProcessJsQueue);
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::ProcessJsQueue) => {
                // Process pending JS evaluations on sidebar
                let mut queue = ZALO_JS_QUEUE.lock().unwrap();
                for (js, tx) in queue.drain(..) {
                    match sidebar.evaluate_script(&js) {
                        Ok(()) => {
                            // wry evaluate_script doesn't return value directly
                            // We use a workaround: JS sets window.__haviz_result
                            let _ = tx.send("ok".to_string());
                        }
                        Err(e) => {
                            let _ = tx.send(format!("error:{}", e));
                        }
                    }
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    let w = new_size.width as f64;
                    let h = new_size.height as f64;
                    let _ = _dashboard.set_bounds(Rect {
                        position: LogicalPosition::new(0.0, 0.0).into(),
                        size: LogicalSize::new(w - SIDEBAR_W, h).into(),
                    });
                    let _ = sidebar.set_bounds(Rect {
                        position: LogicalPosition::new(w - SIDEBAR_W, 0.0).into(),
                        size: LogicalSize::new(SIDEBAR_W, h).into(),
                    });
                }
                _ => {}
            },
            _ => {}
        }
    });
}

async fn start_agent() {
    // Load .env.local
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env.local");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((key, val)) = line.split_once('=') {
                    std::env::set_var(key.trim(), val.trim());
                }
            }
        }
    }

    let config = haviz_agent::config::Config::load();
    let db = std::sync::Arc::new(
        haviz_agent::db::Database::open(&config.db_path).expect("Failed to open DB"),
    );

    // Extended router with Zalo control + screenshot endpoints
    let app = haviz_agent::server::create_router(db)
        .route("/api/screenshot", axum::routing::get(screenshot_handler))
        .route("/api/zalo/search", axum::routing::post(zalo_search_handler))
        .route("/api/zalo/open", axum::routing::post(zalo_open_handler))
        .route("/api/zalo/send", axum::routing::post(zalo_send_handler))
        .route("/api/zalo/conversations", axum::routing::get(zalo_conversations_handler));

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

// === Screenshot ===
async fn screenshot_handler() -> axum::response::Response {
    use std::process::Command;
    let path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Haviz")
        .join("screenshot.png");

    // Use macOS screencapture to capture the Haviz window
    let result = Command::new("screencapture")
        .args(["-l", &get_haviz_window_id(), "-o", path.to_str().unwrap()])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            match std::fs::read(&path) {
                Ok(bytes) => axum::response::Response::builder()
                    .header("Content-Type", "image/png")
                    .body(axum::body::Body::from(bytes))
                    .unwrap(),
                Err(e) => axum::response::Response::builder()
                    .status(500)
                    .body(axum::body::Body::from(format!("Read error: {}", e)))
                    .unwrap(),
            }
        }
        _ => {
            // Fallback: capture entire screen
            let _ = Command::new("screencapture")
                .args(["-x", path.to_str().unwrap()])
                .output();
            match std::fs::read(&path) {
                Ok(bytes) => axum::response::Response::builder()
                    .header("Content-Type", "image/png")
                    .body(axum::body::Body::from(bytes))
                    .unwrap(),
                Err(e) => axum::response::Response::builder()
                    .status(500)
                    .body(axum::body::Body::from(format!("Error: {}", e)))
                    .unwrap(),
            }
        }
    }
}

fn get_haviz_window_id() -> String {
    use std::process::Command;
    // Get Haviz window ID via AppleScript
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to return id of first window of process "haviz_app""#)
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "0".to_string(),
    }
}

// === Zalo Control via JS injection ===

#[derive(serde::Deserialize)]
struct SearchRequest { query: String }

#[derive(serde::Deserialize)]
struct OpenRequest { index: usize }

#[derive(serde::Deserialize)]
struct SendRequest { message: String }

async fn zalo_search_handler(
    axum::extract::Json(req): axum::extract::Json<SearchRequest>,
) -> axum::response::Json<serde_json::Value> {
    let query = req.query.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'");

    // Click search, clear, type query
    let js = format!(r#"
        (function() {{
            var inp = document.querySelector('input[type="text"]');
            if (!inp) return;
            inp.focus();
            inp.value = '';
            inp.dispatchEvent(new Event('input', {{bubbles:true}}));
        }})();
    "#);
    let _ = eval_zalo_js(&js);
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Type each character
    for ch in query.chars() {
        let js = format!(r#"
            (function() {{
                var inp = document.querySelector('input[type="text"]');
                if (!inp) return;
                inp.value += '{}';
                inp.dispatchEvent(new Event('input', {{bubbles:true}}));
            }})();
        "#, ch);
        let _ = eval_zalo_js(&js);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Read results via AX API helper
    let reader_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("helpers").join("zalo_web_reader");

    // The WebView window title should contain "Haviz"
    match std::process::Command::new(&reader_path).arg("haviz_app").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&stdout) {
                axum::response::Json(serde_json::json!({
                    "ok": true,
                    "conversations": data["conversations"],
                }))
            } else {
                axum::response::Json(serde_json::json!({
                    "ok": true,
                    "note": "Search executed, check Zalo sidebar",
                }))
            }
        }
        Err(_) => {
            axum::response::Json(serde_json::json!({
                "ok": true,
                "note": "Search executed, check Zalo sidebar",
            }))
        }
    }
}

async fn zalo_open_handler(
    axum::extract::Json(req): axum::extract::Json<OpenRequest>,
) -> axum::response::Json<serde_json::Value> {
    // Click on conversation by index using JS
    let js = format!(r#"
        (function() {{
            var items = document.querySelectorAll('.truncate');
            var unique = [];
            var seen = new Set();
            for (var el of items) {{
                var t = el.textContent.trim();
                if (t && !seen.has(t) && t.length < 50) {{
                    seen.add(t);
                    unique.push(el);
                }}
            }}
            if ({idx} < unique.length) {{
                unique[{idx}].scrollIntoView({{block:'center'}});
                unique[{idx}].click();
                // Also try parent click
                var p = unique[{idx}].parentElement;
                for (var i = 0; i < 5 && p; i++) {{
                    p.click();
                    p = p.parentElement;
                }}
            }}
        }})();
    "#, idx = req.index.saturating_sub(1));

    let result = eval_zalo_js(&js);
    axum::response::Json(serde_json::json!({
        "ok": true,
        "clicked_index": req.index,
        "result": result.unwrap_or_default(),
    }))
}

async fn zalo_send_handler(
    axum::extract::Json(req): axum::extract::Json<SendRequest>,
) -> axum::response::Json<serde_json::Value> {
    let msg = req.message.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

    // Find chat input and type message
    let js = format!(r#"
        (function() {{
            var input = document.querySelector('[contenteditable="true"]');
            if (!input) input = document.querySelector('textarea');
            if (!input) input = document.querySelector('[class*="input"][class*="chat"]');
            if (!input) return 'input_not_found';
            input.focus();
            input.textContent = "{}";
            input.dispatchEvent(new Event('input', {{bubbles:true}}));
            // Send Enter
            input.dispatchEvent(new KeyboardEvent('keydown', {{key:'Enter', keyCode:13, bubbles:true}}));
            return 'sent';
        }})();
    "#, msg);

    let result = eval_zalo_js(&js);
    axum::response::Json(serde_json::json!({
        "ok": true,
        "message": req.message,
        "result": result.unwrap_or_default(),
    }))
}

async fn zalo_conversations_handler() -> axum::response::Json<serde_json::Value> {
    // Use AX API to read conversations from the WebView
    let reader_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("helpers").join("zalo_web_reader");

    // Try reading from haviz_app process (the WebView)
    match std::process::Command::new(&reader_path).arg("haviz_app").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(data) => axum::response::Json(data),
                Err(_) => axum::response::Json(serde_json::json!({"ok": false, "error": "parse_error"})),
            }
        }
        _ => {
            axum::response::Json(serde_json::json!({"ok": false, "error": "reader_failed"}))
        }
    }
}
