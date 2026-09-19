use crate::http::state::{AppState, RoomStoreState};
use axum::{Json, Router, response::IntoResponse, routing::get};

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

pub(super) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/health", get(health))
}
