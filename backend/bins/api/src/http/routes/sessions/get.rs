use super::dto::{SessionResponse, response_with_room};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{GetSessionForCaller, get_session_for_caller};
use mahfel_domain::SessionId;
use uuid::Uuid;

/// Get a session visible to a room member.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Session", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
    ),
)]
pub(crate) async fn get_session(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = get_session_for_caller(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        GetSessionForCaller {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
        },
    )
    .await?;
    Ok(Json(response_with_room(&state, session).await?))
}
