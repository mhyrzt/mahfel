mod echo;
mod health;
mod status;

use super::error::ApiError;
use axum::Router;

/// JSON envelope fallback for unmatched routes — never an empty/HTML 404.
pub(super) async fn not_found() -> ApiError {
    ApiError::not_found("route not found")
}

/// Versioned route table plus fallback, before transport middleware.
pub(super) fn router(service: &'static str, version: &'static str, node_id: String) -> Router {
    let v1 = status::route(service, version, node_id)
        .merge(health::route())
        .merge(echo::route())
        .fallback(not_found);

    Router::new().nest("/v1", v1).fallback(not_found)
}
