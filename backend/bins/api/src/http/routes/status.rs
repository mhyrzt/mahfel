use super::super::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: String,
}

pub(super) fn route() -> Router<AppState> {
    Router::new().route("/status", get(status))
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        service: state.service,
        version: state.version,
        node_id: state.node_id.clone(),
    })
}
