pub(crate) mod admin;
mod avatar;
pub(crate) mod collaboration;
pub(crate) mod echo;
pub(crate) mod files;
pub(crate) mod health;
pub(crate) mod identity;
pub(crate) mod invite;
pub(crate) mod openapi;
pub(crate) mod participation;
pub(crate) mod recordings;
pub(crate) mod rfc3339;
pub(crate) mod rooms;
pub(crate) mod sessions;
pub(crate) mod status;

pub(crate) use openapi::ApiDoc;

use crate::http::{error::ApiError, state::AppState, ws};
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
pub(crate) fn router(state: AppState) -> Router {
    let v1 = status::route()
        .merge(health::route())
        .merge(echo::route())
        .merge(files::route())
        .merge(collaboration::route())
        .merge(identity::route())
        .merge(admin::route())
        .merge(rooms::route())
        .merge(sessions::route())
        .merge(recordings::route())
        .merge(participation::route())
        .merge(invite::route())
        .merge(ws::route())
        .merge(crate::http::turn::route())
        .route("/openapi.json", get(openapi))
        .fallback(not_found);

    Router::new()
        .nest("/v1", v1)
        .fallback(not_found)
        .with_state(state)
}
