use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: String,
}

pub(super) fn route(service: &'static str, version: &'static str, node_id: String) -> Router {
    let handler = move || {
        let node_id = node_id.clone();
        async move {
            Json(StatusResponse {
                service,
                version,
                node_id,
            })
        }
    };
    Router::new().route("/status", get(handler))
}
