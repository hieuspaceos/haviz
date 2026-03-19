/// WebView construction helpers for the dashboard, Zalo sidebar, and toggle button.
///
/// Layout (sidebar expanded):
/// ┌─────────────────────┬──┬──────────┐
/// │    Dashboard        │▶◀│ Zalo Web │
/// │    (localhost:9999) │  │ Sidebar  │
/// └─────────────────────┴──┴──────────┘
///
/// Layout (sidebar collapsed — default):
/// ┌────────────────────────────────┬──┐
/// │    Dashboard (full width)      │◀│
/// └────────────────────────────────┴──┘
///
/// Session persistence: Each WebView gets its own WebContext with a dedicated
/// data directory under AppData/Roaming/Haviz/webview/.
use std::path::PathBuf;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::{Rect, WebContext, WebView, WebViewBuilder};
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;

use super::ipc::{UserEvent, ZALO_CONVERSATIONS, ZALO_MESSAGES};

/// Width of the Zalo sidebar when expanded.
pub const SIDEBAR_W: f64 = 400.0;
/// Width of the toggle button strip.
pub const TOGGLE_W: f64 = 28.0;

/// Returns persistent data directory under AppData/Roaming/Haviz/webview/{name}
fn webview_data_dir(name: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Haviz")
        .join("webview")
        .join(name)
}

/// Build the dashboard WebView. Starts at full width (sidebar collapsed).
pub fn build_dashboard(window: &Window, window_w: f64, window_h: f64) -> WebView {
    let vite_port = std::env::var("VITE_PORT").unwrap_or_else(|_| "3333".to_string());
    let vite_url = format!("http://localhost:{}", vite_port);
    let dashboard_url = if reqwest::blocking::get(&vite_url).is_ok() {
        vite_url
    } else {
        "http://localhost:9999".to_string()
    };
    println!("Dashboard: {}", dashboard_url);

    let mut ctx = WebContext::new(Some(webview_data_dir("dashboard")));

    WebViewBuilder::with_web_context(&mut ctx)
        .with_url(&dashboard_url)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(window_w - TOGGLE_W, window_h).into(),
        })
        .with_devtools(true)
        .build_as_child(window)
        .expect("Failed to create dashboard WebView")
}

/// Build the toggle button strip between dashboard and sidebar.
/// Sends ToggleSidebar event on click.
pub fn build_toggle_button(
    window: &Window,
    window_w: f64,
    window_h: f64,
    proxy: EventLoopProxy<UserEvent>,
) -> WebView {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><style>
      * { margin:0; padding:0; box-sizing:border-box; }
      body {
        height: 100vh; display: flex; align-items: center; justify-content: center;
        background: #1a1a2e; cursor: pointer; user-select: none;
        border-left: 1px solid #2a2a4a; border-right: 1px solid #2a2a4a;
      }
      body:hover { background: #16213e; }
      .btn {
        color: #7c83ff; font-size: 18px; font-family: system-ui;
        writing-mode: vertical-rl; letter-spacing: 2px;
      }
    </style></head>
    <body onclick="window.ipc.postMessage('toggle')">
      <div class="btn" id="arrow">◀ Zalo</div>
    </body>
    </html>
    "#;

    WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |_req| {
            let _ = proxy.send_event(UserEvent::ToggleSidebar);
        })
        .with_bounds(Rect {
            position: LogicalPosition::new(window_w - TOGGLE_W, 0.0).into(),
            size: LogicalSize::new(TOGGLE_W, window_h).into(),
        })
        .build_as_child(window)
        .expect("Failed to create toggle button")
}

/// Build the Zalo Web sidebar. Starts hidden (0 width, sidebar collapsed).
pub fn build_zalo_sidebar(window: &Window, window_w: f64, _window_h: f64) -> WebView {
    let mut ctx = WebContext::new(Some(webview_data_dir("zalo")));

    let builder = WebViewBuilder::with_web_context(&mut ctx)
        .with_url("https://chat.zalo.me")
        .with_user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/120.0.0.0 Safari/537.36",
        )
        .with_ipc_handler(|req| {
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
                    *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
                }
            }
        })
        .with_bounds(Rect {
            // Start off-screen / zero-width (collapsed by default)
            position: LogicalPosition::new(window_w, 0.0).into(),
            size: LogicalSize::new(0.0, 0.0).into(),
        })
        .with_incognito(false)
        .with_devtools(true);

    #[cfg(target_os = "macos")]
    let builder = {
        let store_id: [u8; 16] = *b"haviz_zalo_web_1";
        builder.with_data_store_identifier(store_id)
    };

    builder
        .build_as_child(window)
        .expect("Failed to create Zalo sidebar WebView")
}

/// Recalculate bounds for all three panels based on sidebar state.
pub fn layout_panels(
    dashboard: &WebView,
    toggle: &WebView,
    sidebar: &WebView,
    w: f64,
    h: f64,
    sidebar_open: bool,
) {
    if sidebar_open {
        let dash_w = w - TOGGLE_W - SIDEBAR_W;
        let _ = dashboard.set_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(dash_w, h).into(),
        });
        let _ = toggle.set_bounds(Rect {
            position: LogicalPosition::new(dash_w, 0.0).into(),
            size: LogicalSize::new(TOGGLE_W, h).into(),
        });
        let _ = sidebar.set_bounds(Rect {
            position: LogicalPosition::new(dash_w + TOGGLE_W, 0.0).into(),
            size: LogicalSize::new(SIDEBAR_W, h).into(),
        });
    } else {
        let dash_w = w - TOGGLE_W;
        let _ = dashboard.set_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(dash_w, h).into(),
        });
        let _ = toggle.set_bounds(Rect {
            position: LogicalPosition::new(dash_w, 0.0).into(),
            size: LogicalSize::new(TOGGLE_W, h).into(),
        });
        let _ = sidebar.set_bounds(Rect {
            position: LogicalPosition::new(w, 0.0).into(),
            size: LogicalSize::new(0.0, 0.0).into(),
        });
    }
}

/// Update the toggle button arrow direction.
pub fn update_toggle_arrow(toggle: &WebView, sidebar_open: bool) {
    let arrow = if sidebar_open { "▶ Zalo" } else { "◀ Zalo" };
    let js = format!(
        "document.getElementById('arrow').textContent = '{}';",
        arrow
    );
    let _ = toggle.evaluate_script(&js);
}
