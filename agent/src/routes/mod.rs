/// Extended route handlers — Zalo control, AI draft, screenshot.
/// These are layered on top of the base router from server::create_router().
use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::db::Database;

pub mod ai_draft;
pub mod screenshot;
pub mod zalo_accumulator;
pub mod zalo_control;
pub mod zalo_db_handlers;
pub mod zalo_scripts;

/// Build the extended router with all non-base routes merged with the base router.
pub fn extended_router(db: Arc<Database>) -> Router {
    let db2 = db.clone(); // clone for DB-backed zalo routes

    // Sub-router for DB-backed Zalo routes — requires State<Arc<Database>>
    let zalo_db_routes: Router = Router::new()
        // /api/zalo/messages — returns accumulated messages from DB (live snapshot fallback)
        .route("/api/zalo/messages", get(zalo_db_handlers::zalo_messages_db_handler))
        // /api/zalo/history — paginated accumulated messages from SQLite
        .route("/api/zalo/history", get(zalo_db_handlers::zalo_history_handler))
        .with_state(db2);

    let router = crate::server::create_router(db)
        .merge(zalo_db_routes)
        .route("/api/ai/draft", post(ai_draft::ai_draft_handler))
        .route("/api/screenshot", get(screenshot::screenshot_handler))
        .route("/api/zalo/conversations", get(zalo_control::zalo_conversations_handler))
        .route("/api/zalo/_messages_callback", get(zalo_control::zalo_messages_callback))
        .route("/api/zalo/search", post(zalo_control::zalo_search_handler))
        .route("/api/zalo/open", post(zalo_control::zalo_open_handler))
        .route("/api/zalo/send", post(zalo_control::zalo_send_handler))
        .route("/api/zalo/search-and-send", post(zalo_control::zalo_search_and_send_handler))
        .route("/api/zalo/debug", get(zalo_control::zalo_debug_handler));

    // Windows-only: Zalo Desktop UI Automation route
    #[cfg(target_os = "windows")]
    let router = router
        .route("/api/zalo/desktop", get(zalo_control::zalo_desktop_handler))
        .route("/api/zalo/clipboard", get(zalo_control::zalo_clipboard_handler));

    router
}
