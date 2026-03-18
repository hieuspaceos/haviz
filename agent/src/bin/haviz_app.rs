/// Haviz App — WebView (chat.zalo.me) + Agent chạy background
///
/// Mở ra user thấy Zalo Web ngay bên trong Haviz
/// Agent đọc tin nhắn qua AX API từ chính WebView này
/// Session persist tự động (WebKit data store)
///
/// Chạy: cargo run --bin haviz_app

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║  Haviz — Revenue Intelligence         ║");
    println!("║  Zalo Web + AI Draft Assistant         ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Data dir for WebView persistence (cookies, localStorage, IndexedDB)
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Haviz")
        .join("webview-data");
    std::fs::create_dir_all(&data_dir).ok();
    println!("WebView data: {}", data_dir.display());

    // Start Agent HTTP server in background thread
    let agent_handle = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            start_agent().await;
        });
    });
    println!("Agent server: http://localhost:9999");

    // Create window with WebView
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Haviz — Zalo")
        .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window");

    // Build WebView with persistent session
    // data_store_identifier = fixed UUID so session persists across app restarts
    let store_id: [u8; 16] = *b"haviz_zalo_web_1"; // 16 bytes, fixed identifier
    let _webview = WebViewBuilder::new()
        .with_url("https://chat.zalo.me")
        .with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
        .with_devtools(true)
        .with_incognito(false)
        .with_data_store_identifier(store_id)
        .build(&window)
        .expect("Failed to create WebView");

    println!("\n✅ Haviz đang chạy!");
    println!("   - Zalo Web trong cửa sổ (login 1 lần, session tự lưu)");
    println!("   - Agent API: http://localhost:9999/api/status");
    println!("   - Web UI: http://localhost:9999 (coming soon)\n");

    // Run event loop (main thread — required by macOS)
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Haviz closing...");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

async fn start_agent() {
    // Reuse config and DB from main agent
    // Load .env.local
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env.local");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
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

    // Start HTTP server (no polling for now — AX API will read from the WebView window)
    let app = haviz_agent::server::create_router(db);
    let addr = format!("0.0.0.0:{}", config.http_port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind HTTP server");
    axum::serve(listener, app).await.expect("HTTP server error");
}
