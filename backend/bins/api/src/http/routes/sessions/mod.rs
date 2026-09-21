pub(crate) mod cancel;
pub(crate) mod dto;
pub(crate) mod end;
pub(crate) mod error;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod schedule;
pub(crate) mod start;

use crate::http::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/rooms/{room_id}/sessions",
            get(list::list_sessions).post(schedule::schedule),
        )
        .route("/sessions/{session_id}", get(get::get_session))
        .route("/sessions/{session_id}/start", post(start::start))
        .route("/sessions/{session_id}/end", post(end::end))
        .route("/sessions/{session_id}/cancel", post(cancel::cancel))
}
