use crate::http::state::{AppState, RoomStoreState};
use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct HealthResponse {
    status: &'static str,
}

#[utoipa::path(
    get,
    path = "/v1/health",
    tag = "status",
    responses((status = 200, description = "Liveness", body = HealthResponse)),
)]
pub(crate) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/health", get(health))
}
