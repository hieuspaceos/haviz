/// Extended route handlers — Zalo control, AI draft, screenshot.
/// These are layered on top of the base router from server::create_router().
use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::db::Database;

pub mod ai_draft;
pub mod screenshot;
pub mod zalo_control;
pub mod zalo_scripts;

/// Build the extended router with all non-base routes merged with the base router.
pub fn extended_router(db: Arc<Database>) -> Router {
    crate::server::create_router(db)
        .route("/api/ai/draft", post(ai_draft::ai_draft_handler))
        .route("/api/screenshot", get(screenshot::screenshot_handler))
        .route("/api/zalo/conversations", get(zalo_control::zalo_conversations_handler))
        .route("/api/zalo/messages", get(zalo_control::zalo_messages_handler))
        .route("/api/zalo/_messages_callback", get(zalo_control::zalo_messages_callback))
        .route("/api/zalo/search", post(zalo_control::zalo_search_handler))
        .route("/api/zalo/open", post(zalo_control::zalo_open_handler))
        .route("/api/zalo/send", post(zalo_control::zalo_send_handler))
        .route("/api/zalo/search-and-send", post(zalo_control::zalo_search_and_send_handler))
}
