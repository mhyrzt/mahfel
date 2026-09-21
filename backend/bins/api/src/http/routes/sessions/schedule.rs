use super::dto::{CreateSessionRequest, SessionResponse};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::sessions::{ScheduleSession, schedule_session};
use mahfel_domain::RoomId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/sessions",
    tag = "sessions",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session scheduled", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
        (status = 409, description = "A live session already exists for the room"),
    ),
)]
pub(crate) async fn schedule(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let session = schedule_session(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        ScheduleSession {
            actor: principal.actor_id,
            room_id: RoomId::from_uuid(room_id),
            scheduled_start_at: body.scheduled_start_at,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(session.into())))
}
