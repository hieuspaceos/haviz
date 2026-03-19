/// Haviz App — Dashboard + Zalo Web Sidebar
///
/// Layout:
/// ┌──────────────────────────┬─────────────┐
/// │    Haviz Dashboard       │  Zalo Web   │
/// │    (localhost:9999)      │  Sidebar    │
/// │    Inbox, AI Drafts      │  (~400px)   │
/// └──────────────────────────┴─────────────┘

use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::Rect;

use haviz_agent::app::ipc::{UserEvent, ZALO_JS_QUEUE};
use haviz_agent::app::webview::{build_dashboard, build_zalo_sidebar};
use haviz_agent::app::app_config::load_dotenv;
use haviz_agent::routes::extended_router;

const WINDOW_W: f64 = 1400.0;
const WINDOW_H: f64 = 900.0;
const SIDEBAR_W: f64 = 400.0;

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║  Haviz — Revenue Intelligence         ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Spawn the HTTP agent server in a background thread
    let _agent = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_agent());
    });

    // Small delay to let the server bind before the WebViews try to load it
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Build event loop + window
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Haviz")
        .with_inner_size(LogicalSize::new(WINDOW_W, WINDOW_H))
        .build(&event_loop)
        .expect("Failed to create window");

    // Build child WebViews
    let dashboard = build_dashboard(&window, WINDOW_W, WINDOW_H);
    let sidebar = build_zalo_sidebar(&window, WINDOW_W, WINDOW_H);

    println!("Haviz đang chạy!");
    println!("   Dashboard: http://localhost:9999");
    println!("   Zalo Web sidebar (phải)\n");

    // Periodic tick to drain the JS evaluation queue
    let proxy2 = proxy.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = proxy2.send_event(UserEvent::ProcessJsQueue);
    });

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::ProcessJsQueue) => {
                // Drain pending JS evaluations onto the Zalo sidebar WebView
                let mut queue = ZALO_JS_QUEUE.lock().unwrap();
                for (js, tx) in queue.drain(..) {
                    match sidebar.evaluate_script(&js) {
                        Ok(()) => {
                            // wry evaluate_script is fire-and-forget; send "ok" as signal
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
                    let _ = dashboard.set_bounds(Rect {
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

/// Load config, open database, build the full router, and start the HTTP server.
async fn start_agent() {
    load_dotenv();

    let config = haviz_agent::config::Config::load();
    let db = std::sync::Arc::new(
        haviz_agent::db::Database::open(&config.db_path).expect("Failed to open DB"),
    );

    let app = extended_router(db);

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
