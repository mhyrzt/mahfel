mod connection;
mod document;
mod fanout;

pub(crate) use fanout::Fanout;

use crate::http::{error::ApiError, state::AppState};
use axum::{
    Router,
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use mahfel_domain::{CollaborationKind, SessionId};
use uuid::Uuid;

pub(super) fn route() -> Router<AppState> {
    Router::new().route("/ws/collaboration/{session_id}/{kind}", get(upgrade))
}

async fn upgrade(
    State(state): State<AppState>,
    Path((session_id, kind)): Path<(Uuid, String)>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let kind = match kind.as_str() {
        "notes" => CollaborationKind::Notes,
        "whiteboard" => CollaborationKind::Whiteboard,
        _ => return Err(ApiError::not_found("document type unavailable")),
    };
    let Some(store) = state.collaboration.clone() else {
        return Err(ApiError::unavailable("collaboration unavailable"));
    };
    let Some(fanout) = state.realtime.collaboration_fanout.clone() else {
        return Err(ApiError::unavailable("collaboration unavailable"));
    };
    let Some(tickets) = state.realtime.auth_tickets.clone() else {
        return Err(ApiError::unavailable(
            "collaboration authentication unavailable",
        ));
    };
    Ok(ws
        .max_message_size(1024 * 1024)
        .max_frame_size(1024 * 1024)
        .on_upgrade(move |socket| {
            connection::run(
                socket,
                SessionId::from_uuid(session_id),
                kind,
                store,
                fanout,
                tickets,
            )
        }))
}
