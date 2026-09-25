pub(crate) mod admission;
pub(crate) mod archive;
pub(crate) mod chat;
pub(crate) mod chat_interactions;
pub(crate) mod chat_read_marker;
pub(crate) mod create;
pub(crate) mod dto;
pub(crate) mod error;
pub(crate) mod get;
pub(crate) mod join;
pub(crate) mod list;
pub(crate) mod lobby;
pub(crate) mod members;
pub(crate) mod membership;
pub(crate) mod update;

use crate::http::state::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/rooms", get(list::list_rooms).post(create::create))
        .route("/rooms/{room_id}", get(get::get_room).patch(update::update))
        .route(
            "/rooms/{room_id}/membership",
            get(membership::own_membership),
        )
        .route("/rooms/{room_id}/join", post(join::join))
        .route("/rooms/{room_id}/lobby", get(lobby::get_lobby))
        .route(
            "/rooms/{room_id}/lobby/{actor_id}/admit",
            post(lobby::admit_principal),
        )
        .route(
            "/rooms/{room_id}/lobby/{actor_id}/reject",
            post(lobby::reject_principal),
        )
        .route(
            "/rooms/{room_id}/messages",
            get(chat::list_messages).post(chat::create_message),
        )
        .route(
            "/rooms/{room_id}/messages/{message_id}",
            patch(chat::edit_message).delete(chat::delete_message),
        )
        .route(
            "/rooms/{room_id}/messages/{message_id}/replies",
            get(chat_interactions::list_replies).post(chat_interactions::create_reply),
        )
        .route(
            "/rooms/{room_id}/messages/{message_id}/mentions",
            get(chat_interactions::list_mentions),
        )
        .route(
            "/rooms/{room_id}/messages/{message_id}/reaction",
            get(chat_interactions::get_reactions)
                .put(chat_interactions::set_reaction)
                .delete(chat_interactions::remove_reaction),
        )
        .route(
            "/rooms/{room_id}/read-marker",
            get(chat_read_marker::get_marker).put(chat_read_marker::advance_marker),
        )
        .route(
            "/rooms/{room_id}/admission",
            get(admission::get_admission).put(admission::set_admission),
        )
        .route("/rooms/{room_id}/archive", post(archive::archive))
        .route(
            "/rooms/{room_id}/members",
            get(members::list).post(members::add),
        )
        .route(
            "/rooms/{room_id}/members/{user_id}",
            delete(members::remove).patch(members::change_role),
        )
}
