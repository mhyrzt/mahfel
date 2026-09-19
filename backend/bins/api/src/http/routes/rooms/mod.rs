pub(crate) mod archive;
pub(crate) mod create;
pub(crate) mod dto;
pub(crate) mod error;
pub(crate) mod get;
pub(crate) mod join;
pub(crate) mod list;
pub(crate) mod members;
pub(crate) mod update;

use crate::http::state::{AppState, RoomStoreState};
use axum::{
    Router,
    routing::{delete, get, post},
};

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new()
        .route(
            "/rooms",
            get(list::list_rooms::<R>).post(create::create::<R>),
        )
        .route(
            "/rooms/{room_id}",
            get(get::get_room::<R>).patch(update::update::<R>),
        )
        .route("/rooms/{room_id}/join", post(join::join::<R>))
        .route("/rooms/{room_id}/archive", post(archive::archive::<R>))
        .route("/rooms/{room_id}/members", post(members::add::<R>))
        .route(
            "/rooms/{room_id}/members/{user_id}",
            delete(members::remove::<R>),
        )
}
