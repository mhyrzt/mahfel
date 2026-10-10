use super::dto::{SessionResponse, SessionStatusDto};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedPagination, ValidatedPath},
    pagination::Paginated,
    state::AppState,
};
use axum::{
    Json,
    extract::{Query, State},
};
use mahfel_application::sessions::{ListSessions, list_sessions as run_list_sessions};
use mahfel_domain::{RoomId, SessionStatus};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub(crate) struct SessionListFilter {
    status: Option<SessionStatusDto>,
}

/// List a room's sessions for a member, newest first.
#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/sessions",
    tag = "sessions",
    params(
        ("room_id" = String, Path, description = "Room identifier (UUID)"),
        ("page" = Option<u64>, Query, description = "Page number, 1-based"),
        ("per_page" = Option<u64>, Query, description = "Items per page, 1-100"),
        ("status" = Option<SessionStatusDto>, Query, description = "Optional lifecycle status filter"),
    ),
    responses(
        (status = 200, description = "Room sessions, newest first", body = Paginated<SessionResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
    ),
)]
pub(crate) async fn list_sessions(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedPagination(pagination): ValidatedPagination,
    Query(filter): Query<SessionListFilter>,
) -> Result<Json<Paginated<SessionResponse>>, ApiError> {
    let sessions = run_list_sessions(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        ListSessions {
            actor: principal.actor_id,
            room_id: RoomId::from_uuid(room_id),
            limit: pagination.limit(),
            offset: pagination.offset(),
            status: filter.status.map(|status| match status {
                SessionStatusDto::Scheduled => SessionStatus::Scheduled,
                SessionStatusDto::Live => SessionStatus::Live,
                SessionStatusDto::Ended => SessionStatus::Ended,
                SessionStatusDto::Cancelled => SessionStatus::Cancelled,
            }),
        },
    )
    .await?;
    let room = state
        .rooms
        .get(RoomId::from_uuid(room_id))
        .await
        .map_err(|_| ApiError::internal("room lookup failed"))?
        .ok_or_else(|| ApiError::internal("room missing"))?;
    Ok(Json(Paginated::new(
        sessions
            .into_iter()
            .map(|session| SessionResponse::from_session(session, room.title.as_str()))
            .collect(),
        pagination,
        None,
    )))
}
