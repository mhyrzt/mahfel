use super::dto::SessionResponse;
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::ValidatedPath,
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{StartSession, start_session};
use mahfel_domain::SessionId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/start",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Session is live", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
        (status = 409, description = "Invalid transition or a live session already exists"),
    ),
)]
pub(crate) async fn start<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = start_session(
        &state.sessions,
        &state.rooms,
        StartSession {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
        },
    )
    .await?;
    Ok(Json(session.into()))
}
