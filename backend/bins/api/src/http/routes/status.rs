use crate::http::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: String,
}

pub(crate) fn route() -> Router<AppState> {
    Router::new().route("/status", get(status))
}

#[utoipa::path(
    get,
    path = "/v1/status",
    summary = "Get service status",
    tag = "status",
    security(()),
    responses((status = 200, description = "Service status", body = StatusResponse)),
)]
pub(crate) async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        service: state.service,
        version: state.version,
        node_id: state.node_id.clone(),
    })
}
