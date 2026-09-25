pub(crate) mod create;
pub(crate) mod dto;
pub(crate) mod error;
pub(crate) mod list;
pub(crate) mod manage;
pub(crate) mod redeem;
pub(crate) mod status;

use crate::http::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/rooms/{room_id}/invite-links",
            get(list::list_for_room).post(create::create_for_room),
        )
        .route(
            "/sessions/{session_id}/invite-links",
            get(list::list_for_session).post(create::create_for_session),
        )
        .route("/invite-links/redeem", post(redeem::redeem))
        .route(
            "/invite-links/requests/{request_id}",
            get(status::request_status),
        )
        .route("/invite-links/{link_id}/revoke", post(manage::revoke))
        .route("/invite-links/{link_id}/rotate", post(manage::rotate))
}
