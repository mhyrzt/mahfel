use super::dto::{CreateSessionRequest, SessionResponse, response_with_room};
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
    summary = "Schedule a session",
    tag = "sessions",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session scheduled", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller lacks manage_session"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
        (status = 409, description = "A live session already exists for the room"),
        (status = 422, description = "Invalid title or advisory duration"),
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
            principal,
            room_id: RoomId::from_uuid(room_id),
            title: body.title,
            scheduled_start_at: body.scheduled_start_at,
            scheduled_duration_minutes: body.scheduled_duration_minutes,
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(response_with_room(&state, session).await?),
    ))
}
