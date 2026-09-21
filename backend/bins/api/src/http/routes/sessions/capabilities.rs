use super::dto::SessionCapabilitiesResponse;
use crate::http::{
    auth, auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{GetSessionForCaller, get_session_for_caller};
use mahfel_domain::SessionId;
use uuid::Uuid;

/// Capabilities the caller holds in the session's room.
///
/// Scoped to the caller: the response never describes another participant's
/// privileges, and a session the caller cannot see is a `404`.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/capabilities",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Caller's capabilities in the session's room", body = SessionCapabilitiesResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
    ),
)]
pub(crate) async fn get_capabilities(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionCapabilitiesResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let session = get_session_for_caller(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        GetSessionForCaller {
            actor: principal.actor_id,
            session_id,
        },
    )
    .await?;
    let capabilities =
        auth::capabilities_in_room(state.rooms.as_ref(), &principal, session.room_id).await?;
    Ok(Json(SessionCapabilitiesResponse::from_set(&capabilities)))
}
