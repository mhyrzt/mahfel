pub(crate) mod access;
pub(crate) mod cancel;
pub(crate) mod capabilities;
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
        .route(
            "/sessions/{session_id}/capabilities",
            get(capabilities::get_capabilities),
        )
        .route("/sessions/{session_id}/start", post(start::start))
        .route("/sessions/{session_id}/end", post(end::end))
        .route("/sessions/{session_id}/cancel", post(cancel::cancel))
        .route(
            "/sessions/{session_id}/access",
            get(access::get_access).put(access::set_access),
        )
        .route("/sessions/{session_id}/lobby", get(access::get_lobby))
        .route(
            "/sessions/{session_id}/lobby/{actor_id}/admit",
            post(access::admit_principal),
        )
        .route(
            "/sessions/{session_id}/lobby/{actor_id}/reject",
            post(access::reject_principal),
        )
}
