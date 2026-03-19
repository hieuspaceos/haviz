/// Haviz App — Dashboard + Zalo Web Sidebar (collapsible)
///
/// Default: sidebar collapsed, toggle button on right edge.
/// Click toggle to expand/collapse Zalo Web sidebar.

use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;

use haviz_agent::app::ipc::{UserEvent, ZALO_JS_QUEUE};
use haviz_agent::app::webview::{
    build_dashboard, build_toggle_button, build_zalo_sidebar,
    layout_panels, update_toggle_arrow,
};
use haviz_agent::app::app_config::load_dotenv;
use haviz_agent::routes::extended_router;

const WINDOW_W: f64 = 1400.0;
const WINDOW_H: f64 = 900.0;

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║  Haviz — Revenue Intelligence         ║");
    println!("╚═══════════════════════════════════════╝\n");

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

    // Build panels: dashboard (full width), toggle strip, sidebar (hidden)
    let dashboard = build_dashboard(&window, WINDOW_W, WINDOW_H);
    let toggle = build_toggle_button(&window, WINDOW_W, WINDOW_H, proxy.clone());
    let sidebar = build_zalo_sidebar(&window, WINDOW_W, WINDOW_H);

    // Sidebar starts collapsed
    let mut sidebar_open = false;

    println!("Haviz đang chạy!");
    println!("   Dashboard: http://localhost:9999");
    println!("   Click ◀ to open Zalo sidebar\n");

    // Periodic tick to drain the JS evaluation queue
    let proxy2 = proxy.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = proxy2.send_event(UserEvent::ProcessJsQueue);
    });

    // Track window size for resize events
    let mut win_w = WINDOW_W;
    let mut win_h = WINDOW_H;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::ProcessJsQueue) => {
                let mut queue = ZALO_JS_QUEUE.lock().unwrap();
                for (js, tx) in queue.drain(..) {
                    match sidebar.evaluate_script(&js) {
                        Ok(()) => { let _ = tx.send("ok".to_string()); }
                        Err(e) => { let _ = tx.send(format!("error:{}", e)); }
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleSidebar) => {
                sidebar_open = !sidebar_open;
                layout_panels(&dashboard, &toggle, &sidebar, win_w, win_h, sidebar_open);
                update_toggle_arrow(&toggle, sidebar_open);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    win_w = new_size.width as f64;
                    win_h = new_size.height as f64;
                    layout_panels(&dashboard, &toggle, &sidebar, win_w, win_h, sidebar_open);
                }
                _ => {}
            },
            _ => {}
        }
    });
}

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
