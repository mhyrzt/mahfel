pub(crate) mod cancel;
pub(crate) mod dto;
pub(crate) mod end;
pub(crate) mod error;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod schedule;
pub(crate) mod start;

use crate::http::state::{AppState, RoomStoreState};
use axum::{
    Router,
    routing::{get, post},
};

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new()
        .route(
            "/rooms/{room_id}/sessions",
            get(list::list_sessions::<R>).post(schedule::schedule::<R>),
        )
        .route("/sessions/{session_id}", get(get::get_session::<R>))
        .route("/sessions/{session_id}/start", post(start::start::<R>))
        .route("/sessions/{session_id}/end", post(end::end::<R>))
        .route("/sessions/{session_id}/cancel", post(cancel::cancel::<R>))
}
