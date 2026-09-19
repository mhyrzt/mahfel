mod echo;
mod health;
mod identity;
mod rooms;
mod status;

use crate::http::{
    error::ApiError,
    state::{AppState, RoomStoreState},
    ws,
};
use axum::Router;

/// JSON envelope fallback for unmatched routes — never an empty/HTML 404.
pub(super) async fn not_found() -> ApiError {
    ApiError::not_found("route not found")
}

/// Versioned route table plus fallback, before transport middleware.
pub(super) fn router<R: RoomStoreState>(state: AppState<R>) -> Router {
    let v1 = status::route::<R>()
        .merge(health::route::<R>())
        .merge(echo::route::<R>())
        .merge(identity::route::<R>())
        .merge(rooms::route::<R>())
        .merge(ws::route::<R>())
        .fallback(not_found);

    Router::new()
        .nest("/v1", v1)
        .fallback(not_found)
        .with_state(state)
}
