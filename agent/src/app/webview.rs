/// WebView construction helpers for the dashboard and Zalo sidebar panes.
///
/// Both panes are child WebViews attached to the same tao Window. The sidebar
/// wires up the IPC handler that populates ipc::ZALO_MESSAGES / ZALO_CONVERSATIONS.
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::window::Window;
use wry::{Rect, WebView, WebViewBuilder};
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;

use super::ipc::{ZALO_CONVERSATIONS, ZALO_MESSAGES};

const SIDEBAR_W: f64 = 400.0;

/// Build the left-side dashboard WebView.
/// Uses Vite dev server on port 3000 when available, otherwise falls back to
/// the agent's built static files served on port 9999.
pub fn build_dashboard(window: &Window, window_w: f64, window_h: f64) -> WebView {
    let dashboard_url = if reqwest::blocking::get("http://localhost:3000").is_ok() {
        "http://localhost:3000"
    } else {
        "http://localhost:9999"
    };
    println!("Dashboard: {}", dashboard_url);

    WebViewBuilder::new()
        .with_url(dashboard_url)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(window_w - SIDEBAR_W, window_h).into(),
        })
        .with_devtools(true)
        .build_as_child(window)
        .expect("Failed to create dashboard WebView")
}

/// Build the right-side Zalo Web sidebar WebView.
/// Registers the IPC handler that receives JSON from JS running inside the sidebar.
pub fn build_zalo_sidebar(window: &Window, window_w: f64, window_h: f64) -> WebView {
    let builder = WebViewBuilder::new()
        .with_url("https://chat.zalo.me")
        .with_user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/120.0.0.0 Safari/537.36",
        )
        .with_ipc_handler(|req| {
            // JS calls: window.ipc.postMessage(JSON.stringify(data))
            let body = req.body();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(msg_type) = parsed.get("type").and_then(|t| t.as_str()) {
                    match msg_type {
                        "conversations" => {
                            let data =
                                parsed.get("data").cloned().unwrap_or(serde_json::json!([]));
                            *ZALO_CONVERSATIONS.lock().unwrap() = Some(data);
                        }
                        _ => {
                            *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
                        }
                    }
                } else {
                    // No type field — treat as messages (backward compat)
                    *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
                }
            }
        })
        .with_bounds(Rect {
            position: LogicalPosition::new(window_w - SIDEBAR_W, 0.0).into(),
            size: LogicalSize::new(SIDEBAR_W, window_h).into(),
        })
        .with_incognito(false)
        .with_devtools(true);

    // Persistent data store (macOS WKWebView only — Windows uses default profile)
    #[cfg(target_os = "macos")]
    let builder = {
        let store_id: [u8; 16] = *b"haviz_zalo_web_1";
        builder.with_data_store_identifier(store_id)
    };

    builder
        .build_as_child(window)
        .expect("Failed to create Zalo sidebar WebView")
}
