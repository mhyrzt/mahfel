use crate::http::state::{AppState, RoomStoreState};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: String,
}

pub(super) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/status", get(status::<R>))
}

async fn status<R: RoomStoreState>(State(state): State<AppState<R>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        service: state.service,
        version: state.version,
        node_id: state.node_id.clone(),
    })
}
