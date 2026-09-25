use super::dto::SessionResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{ListSessions, list_sessions as run_list_sessions};
use mahfel_domain::RoomId;
use uuid::Uuid;

/// List a room's sessions for a member, newest first.
#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/sessions",
    tag = "sessions",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Room sessions, newest first", body = [SessionResponse]),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
    ),
)]
pub(crate) async fn list_sessions(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<Vec<SessionResponse>>, ApiError> {
    let sessions = run_list_sessions(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        ListSessions {
            actor: principal.actor_id,
            room_id: RoomId::from_uuid(room_id),
        },
    )
    .await?;
    let room = state
        .rooms
        .get(RoomId::from_uuid(room_id))
        .await
        .map_err(|_| ApiError::internal("room lookup failed"))?
        .ok_or_else(|| ApiError::internal("room missing"))?;
    Ok(Json(
        sessions
            .into_iter()
            .map(|session| SessionResponse::from_session(session, room.title.as_str()))
            .collect(),
    ))
}
