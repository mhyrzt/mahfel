pub(crate) mod correct;
pub(crate) mod dto;
pub(crate) mod error;
pub(crate) mod participant;
pub(crate) mod session;

use crate::http::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/sessions/{session_id}/participation",
            get(session::session_summary),
        )
        .route(
            "/sessions/{session_id}/participants/{participant_id}/participation",
            get(participant::participant_summary),
        )
        .route(
            "/participation/intervals/{interval_id}/correct",
            post(correct::correct),
        )
}
