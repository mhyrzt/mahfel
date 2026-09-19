pub(crate) mod echo;
pub(crate) mod health;
pub(crate) mod identity;
pub(crate) mod openapi;
pub(crate) mod rooms;
pub(crate) mod status;

pub(crate) use openapi::ApiDoc;

use crate::http::{
    error::ApiError,
    state::{AppState, RoomStoreState},
    ws,
};
use axum::{Json, Router, routing::get};
use utoipa::OpenApi;

/// JSON envelope fallback for unmatched routes — never an empty/HTML 404.
pub(crate) async fn not_found() -> ApiError {
    ApiError::not_found("route not found")
}

/// Machine-readable API contract (ADR 0015).
pub(crate) async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Versioned route table plus fallback, before transport middleware.
pub(crate) fn router<R: RoomStoreState>(state: AppState<R>) -> Router {
    let v1 = status::route::<R>()
        .merge(health::route::<R>())
        .merge(echo::route::<R>())
        .merge(identity::route::<R>())
        .merge(rooms::route::<R>())
        .merge(ws::route::<R>())
        .route("/openapi.json", get(openapi))
        .fallback(not_found);

    Router::new()
        .nest("/v1", v1)
        .fallback(not_found)
        .with_state(state)
}
