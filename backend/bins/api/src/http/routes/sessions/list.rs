use super::dto::SessionResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{ListSessions, list_sessions as run_list_sessions};
use mahfel_domain::RoomId;
use uuid::Uuid;

/// Manager-scoped in this task; member visibility arrives with the capability
/// model (M4-04). A room holds a bounded number of occurrences, so the list is
/// returned whole rather than paginated.
#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/sessions",
    tag = "sessions",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Room sessions, newest first", body = [SessionResponse]),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
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
    Ok(Json(
        sessions.into_iter().map(SessionResponse::from).collect(),
    ))
}
