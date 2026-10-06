pub(crate) mod dto;
pub(crate) mod handlers;
pub(crate) mod reactions;

use crate::http::AppState;
use axum::{
    Router,
    routing::{get, post},
};
pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/sessions/{session_id}/interactions/reactions",
            post(reactions::react),
        )
        .route(
            "/sessions/{session_id}/interactions",
            get(handlers::snapshot),
        )
        .route(
            "/sessions/{session_id}/interactions/commands",
            post(handlers::command),
        )
        .route(
            "/sessions/{session_id}/interactions/activities/{activity_id}/voters",
            get(handlers::voters),
        )
}
