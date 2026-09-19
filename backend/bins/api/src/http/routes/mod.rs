mod echo;
mod health;
mod identity;
mod status;

use super::{error::ApiError, state::AppState};
use axum::Router;

/// JSON envelope fallback for unmatched routes — never an empty/HTML 404.
pub(super) async fn not_found() -> ApiError {
    ApiError::not_found("route not found")
}

/// Versioned route table plus fallback, before transport middleware.
pub(super) fn router(state: AppState) -> Router {
    let v1 = status::route()
        .merge(health::route())
        .merge(echo::route())
        .merge(identity::route())
        .fallback(not_found);

    Router::new()
        .nest("/v1", v1)
        .fallback(not_found)
        .with_state(state)
}
