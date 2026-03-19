/// Route handler for POST /api/ai/draft — generates an AI reply suggestion via Groq.
use axum::{response::Json, extract};
use serde::Deserialize;

use crate::ai::ChatMessage;

#[derive(Deserialize)]
pub struct AiDraftRequest {
    pub messages: Vec<ChatMessage>,
    pub org_context: Option<String>,
}

/// POST /api/ai/draft
/// Expects JSON body matching AiDraftRequest. Returns the generated draft or an error.
pub async fn ai_draft_handler(
    extract::Json(req): extract::Json<AiDraftRequest>,
) -> Json<serde_json::Value> {
    let api_key = match std::env::var("GROQ_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            return Json(serde_json::json!({
                "ok": false,
                "error": "GROQ_API_KEY not set. Add it to .env.local",
            }));
        }
    };

    if req.messages.is_empty() {
        return Json(serde_json::json!({
            "ok": false,
            "error": "No messages provided",
        }));
    }

    match crate::ai::generate_draft(&api_key, &req.messages, req.org_context.as_deref()).await {
        Ok(draft) => Json(serde_json::json!({ "ok": true, "draft": draft })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e })),
    }
}
