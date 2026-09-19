use axum::{Json, Router, response::IntoResponse, routing::get};

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

pub(super) fn route() -> Router {
    Router::new().route("/health", get(health))
}
