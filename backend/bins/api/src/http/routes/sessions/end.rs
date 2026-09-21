use super::dto::SessionResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{EndSession, end_session};
use mahfel_domain::SessionId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/end",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Session ended", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
        (status = 409, description = "Invalid transition"),
    ),
)]
pub(crate) async fn end(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = end_session(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        EndSession {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
        },
    )
    .await?;
    Ok(Json(session.into()))
}
