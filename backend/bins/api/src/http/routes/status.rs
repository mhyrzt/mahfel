use crate::http::state::{AppState, RoomStoreState};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: String,
}

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/status", get(status::<R>))
}

#[utoipa::path(
    get,
    path = "/v1/status",
    tag = "status",
    responses((status = 200, description = "Service status", body = StatusResponse)),
)]
pub(crate) async fn status<R: RoomStoreState>(
    State(state): State<AppState<R>>,
) -> Json<StatusResponse> {
    Json(StatusResponse {
        service: state.service,
        version: state.version,
        node_id: state.node_id.clone(),
    })
}
