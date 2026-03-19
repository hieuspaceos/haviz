/// Bearer token auth middleware for axum routes.
///
/// Reads AGENT_AUTH_TOKEN from config; if set, validates the incoming
/// `Authorization: Bearer <token>` header. Returns 401 JSON on failure.
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{Json, Response},
};
use serde_json::json;

use crate::config::Config;

/// Axum middleware: validates Bearer token if AGENT_AUTH_TOKEN is configured.
/// If token is not configured, all requests pass through (open/dev mode).
pub async fn require_auth(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let config = Config::load();

    // If no token configured, skip auth (open/dev mode)
    let expected = match config.agent_auth_token {
        None => return Ok(next.run(request).await),
        Some(ref t) if t.is_empty() => return Ok(next.run(request).await),
        Some(ref t) => t.clone(),
    };

    let token = extract_bearer(request.headers().get(header::AUTHORIZATION));

    if token.as_deref() == Some(expected.as_str()) {
        Ok(next.run(request).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "Unauthorized" })),
        ))
    }
}

/// Extract the raw token string from `Authorization: Bearer <token>`.
fn extract_bearer(header_val: Option<&header::HeaderValue>) -> Option<String> {
    let val = header_val?.to_str().ok()?;
    val.strip_prefix("Bearer ").map(|s| s.to_string())
}
