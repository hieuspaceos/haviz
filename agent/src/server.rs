use axum::{
    extract::{Path, Query, State},
    http::{header, Method, StatusCode},
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::auth::require_auth;
use crate::db::Database;
use crate::platform::macos::automation;

type Db = Arc<Database>;

pub fn create_router(db: Db) -> Router {
    // Serve Svelte build output, fallback to agent/static/
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let static_dir = if project_root.join("apps/web/dist").exists() {
        project_root.join("apps/web/dist")
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
    };

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3333".parse().unwrap(),
            "http://localhost:9999".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Protected routes — require Bearer token when AGENT_AUTH_TOKEN is set
    let protected = Router::new()
        .route("/api/conversations", get(list_conversations))
        .route("/api/conversations/{id}/messages", get(get_messages))
        .route("/api/conversations/{id}/read", post(mark_read))
        .route("/api/messages/recent", get(recent_messages))
        .route("/api/send", post(send_message))
        .route("/api/drafts", get(list_drafts))
        .route("/api/drafts/{id}/approve", post(approve_draft))
        .route("/api/drafts/{id}/reject", post(reject_draft))
        .route("/api/templates", get(list_templates))
        .route("/api/templates", post(create_template))
        .layer(middleware::from_fn(require_auth))
        .with_state(db);

    Router::new()
        // Public — no auth
        .route("/api/status", get(status))
        // Protected API routes
        .merge(protected)
        .fallback_service(tower_http::services::ServeDir::new(static_dir))
        .layer(cors)
}

// === Status ===

async fn status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "agent": "haviz",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// === Conversations ===

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
}

async fn list_conversations(
    State(db): State<Db>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let convs = db.get_conversations(q.limit.unwrap_or(50)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "conversations": convs })))
}

// === Messages ===

async fn get_messages(
    State(db): State<Db>,
    Path(id): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let msgs = db.get_messages(&id, q.limit.unwrap_or(100)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "messages": msgs })))
}

async fn recent_messages(
    State(db): State<Db>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let msgs = db.get_recent_messages(q.limit.unwrap_or(20)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "messages": msgs })))
}

async fn mark_read(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    db.mark_conversation_read(&id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// === Send ===

#[derive(Deserialize)]
struct SendRequest {
    to: String,
    message: String,
}

async fn send_message(
    Json(req): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: integrate SafetyEngine.check() before send.
    // Example (SafetyEngine would live in app State as Arc<Mutex<SafetyEngine>>):
    //
    //   let mut engine = state.safety_engine.lock().await;
    //   match engine.check(&req.to, &req.message) {
    //       SafetyResult::Allow => { /* proceed */ }
    //       SafetyResult::Queue { reason, send_at } => {
    //           return Ok(Json(json!({ "ok": false, "queued": true,
    //               "reason": reason, "send_at": send_at.to_rfc3339() })));
    //       }
    //       SafetyResult::Block { reason } => {
    //           return Err(StatusCode::FORBIDDEN);  // or json error body
    //       }
    //   }
    //   // After send: engine.record_send_result(ok, &req.to, &req.message);

    // Run AppleScript in blocking task (it takes several seconds)
    let to = req.to.clone();
    let msg = req.message.clone();
    let result = tokio::task::spawn_blocking(move || {
        automation::send_message_zalo_desktop(&to, &msg)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true, "to": req.to, "message": req.message }))),
        Err(e) => Ok(Json(serde_json::json!({ "ok": false, "error": e }))),
    }
}

// === Drafts ===

async fn list_drafts(
    State(db): State<Db>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let drafts = db.get_pending_drafts().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "drafts": drafts })))
}

#[derive(Deserialize)]
struct ApproveRequest {
    to: Option<String>,
    edited_content: Option<String>,
}

async fn approve_draft(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get draft
    let drafts = db.get_pending_drafts().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let draft = drafts.iter().find(|d| d.id == id);
    let draft = match draft {
        Some(d) => d.clone(),
        None => return Ok(Json(serde_json::json!({ "ok": false, "error": "Draft not found" }))),
    };

    let content = req.edited_content.as_deref().unwrap_or(&draft.content);

    // Send if recipient provided
    if let Some(to) = &req.to {
        let to = to.clone();
        let content = content.to_string();
        let result = tokio::task::spawn_blocking(move || {
            automation::send_message_zalo_desktop(&to, &content)
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Err(e) = result {
            return Ok(Json(serde_json::json!({ "ok": false, "error": e })));
        }
    }

    db.update_draft_status(&id, "approved").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "draft_id": id, "status": "approved" })))
}

async fn reject_draft(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    db.update_draft_status(&id, "rejected").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "draft_id": id, "status": "rejected" })))
}

// === Templates ===

async fn list_templates(
    State(db): State<Db>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let templates = db.get_templates().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "templates": templates })))
}

#[derive(Deserialize)]
struct CreateTemplateRequest {
    name: String,
    content: String,
    category: Option<String>,
    match_patterns: Option<Vec<String>>,
}

async fn create_template(
    State(db): State<Db>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let patterns = req.match_patterns.unwrap_or_default();
    let id = db
        .insert_template(&req.name, &req.content, req.category.as_deref(), &patterns)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id })))
}
