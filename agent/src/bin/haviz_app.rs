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

    // Left: Dashboard — use Vite dev server if running, otherwise Agent static
    let dashboard_url = if reqwest::blocking::get("http://localhost:3000").is_ok() {
        "http://localhost:3000" // Vite dev server (hot reload)
    } else {
        "http://localhost:9999" // Production (built static files)
    };
    println!("Dashboard: {}", dashboard_url);

    let _dashboard = WebViewBuilder::new()
        .with_url(dashboard_url)
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
// Requires: System Settings → Privacy & Security → Screen Recording → enable Terminal
async fn screenshot_handler() -> axum::response::Response {
    use std::process::Command;
    let path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Haviz")
        .join("screenshot.png");

    // screencapture -x = no sound, captures the screen
    // -w = interactive window select (but we want automatic)
    let _ = Command::new("screencapture")
        .args(["-x", "-o", path.to_str().unwrap()])
        .output();

    match std::fs::read(&path) {
        Ok(bytes) => axum::response::Response::builder()
            .header("Content-Type", "image/png")
            .body(axum::body::Body::from(bytes))
            .unwrap(),
        Err(e) => axum::response::Response::builder()
            .status(500)
            .header("Content-Type", "text/plain")
            .body(axum::body::Body::from(format!(
                "Screenshot failed: {}. Add Terminal to Screen Recording in System Settings.", e
            )))
            .unwrap(),
    }
}

// === Zalo Control via AppleScript ===
// All interactions use OS-level AppleScript (click coordinates, clipboard paste)
// This is the SAME approach validated in agent-prototype.js — works with Zalo's React events

fn run_osascript(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(serde::Deserialize)]
struct SearchRequest { query: String }

#[derive(serde::Deserialize)]
struct OpenRequest { index: usize }

#[derive(serde::Deserialize)]
struct SendMsgRequest { message: String }

async fn zalo_search_handler(
    axum::extract::Json(req): axum::extract::Json<SearchRequest>,
) -> axum::response::Json<serde_json::Value> {
    let query = req.query.clone();

    // Run in blocking task (AppleScript takes seconds)
    let result = tokio::task::spawn_blocking(move || {
        // Step 1: Focus Haviz window, click on Zalo sidebar search area
        // The Zalo sidebar is on the right side of the Haviz window
        let focus = run_osascript(&format!(r#"
            tell application "System Events"
                tell process "haviz_app"
                    set frontmost to true
                    set winPos to position of window 1
                    set winSize to size of window 1
                end tell
                -- Click search area in Zalo sidebar (right portion)
                -- Sidebar starts at window_width - 400, search box is near top
                set xSearch to (item 1 of winPos) + (item 1 of winSize) - 200
                set ySearch to (item 2 of winPos) + 85
                click at {{xSearch, ySearch}}
                delay 0.3
                -- Select all + delete (clear existing search)
                keystroke "a" using command down
                delay 0.1
                key code 51
                delay 0.2
            end tell
        "#));

        if let Err(e) = focus {
            return Err(format!("Focus failed: {}", e));
        }

        // Step 2: Type search query via clipboard (handles Vietnamese)
        let _ = run_osascript(&format!(r#"set the clipboard to "{}""#, esc(&query)));
        let _ = run_osascript(r#"
            tell application "System Events"
                keystroke "v" using command down
                delay 1.5
            end tell
        "#);

        Ok("search_done".to_string())
    }).await.unwrap_or(Err("task failed".to_string()));

    match result {
        Ok(_) => axum::response::Json(serde_json::json!({
            "ok": true,
            "query": req.query,
            "note": "Search typed. Results visible in Zalo sidebar.",
        })),
        Err(e) => axum::response::Json(serde_json::json!({
            "ok": false,
            "error": e,
        })),
    }
}

async fn zalo_open_handler(
    axum::extract::Json(req): axum::extract::Json<OpenRequest>,
) -> axum::response::Json<serde_json::Value> {
    let index = req.index;

    let result = tokio::task::spawn_blocking(move || {
        // Click on the Nth item in Zalo sidebar conversation/search results
        // Each item is ~72px tall, first item starts at ~140px from top of window
        let item_height = 72;
        let first_item_y = 140;
        let item_y = first_item_y + ((index - 1) * item_height) + (item_height / 2);

        let script = format!(r#"
            tell application "System Events"
                tell process "haviz_app"
                    set frontmost to true
                    set winPos to position of window 1
                    set winSize to size of window 1
                end tell
                -- Click on item in Zalo sidebar
                set xClick to (item 1 of winPos) + (item 1 of winSize) - 200
                set yClick to (item 2 of winPos) + {}
                click at {{xClick, yClick}}
                delay 0.5
            end tell
        "#, item_y);

        run_osascript(&script)
    }).await.unwrap_or(Err("task failed".to_string()));

    match result {
        Ok(_) => axum::response::Json(serde_json::json!({
            "ok": true,
            "clicked_index": index,
        })),
        Err(e) => axum::response::Json(serde_json::json!({
            "ok": false,
            "error": e,
        })),
    }
}

async fn zalo_send_handler(
    axum::extract::Json(req): axum::extract::Json<SendMsgRequest>,
) -> axum::response::Json<serde_json::Value> {
    let message = req.message.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Click on chat input area (bottom of Zalo sidebar)
        let click_input = run_osascript(r#"
            tell application "System Events"
                tell process "haviz_app"
                    set frontmost to true
                    set winPos to position of window 1
                    set winSize to size of window 1
                end tell
                -- Click chat input (bottom of Zalo sidebar)
                set xInput to (item 1 of winPos) + (item 1 of winSize) - 200
                set yInput to (item 2 of winPos) + (item 2 of winSize) - 50
                click at {xInput, yInput}
                delay 0.3
            end tell
        "#);

        if let Err(e) = click_input {
            return Err(format!("Click input failed: {}", e));
        }

        // Paste message via clipboard (handles Vietnamese + emoji)
        let _ = run_osascript(&format!(r#"set the clipboard to "{}""#, esc(&message)));
        let _ = run_osascript(r#"
            tell application "System Events"
                keystroke "v" using command down
                delay 0.3
                key code 36
            end tell
        "#);

        Ok("sent".to_string())
    }).await.unwrap_or(Err("task failed".to_string()));

    match result {
        Ok(_) => axum::response::Json(serde_json::json!({
            "ok": true,
            "message": req.message,
        })),
        Err(e) => axum::response::Json(serde_json::json!({
            "ok": false,
            "error": e,
        })),
    }
}

async fn zalo_conversations_handler() -> axum::response::Json<serde_json::Value> {
    // Use AX API to read conversations from the Haviz WebView
    let reader_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("helpers").join("zalo_web_reader");

    let result = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&reader_path).arg("haviz_app").output()
    }).await;

    match result {
        Ok(Ok(output)) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(data) => axum::response::Json(data),
                Err(_) => axum::response::Json(serde_json::json!({"ok": false, "error": "parse_error"})),
            }
        }
        _ => {
            // AX API might not find "haviz_app" as browser name
            // Fallback: return empty with helpful message
            axum::response::Json(serde_json::json!({
                "ok": false,
                "conversations": [],
                "note": "AX API reader could not find Haviz WebView. Conversations visible in sidebar.",
            }))
        }
    }
}
