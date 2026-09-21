use super::dto::SessionParticipationResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::participation::{SessionParticipation, session_participation};
use mahfel_domain::SessionId;
use time::OffsetDateTime;
use uuid::Uuid;

/// Per-participant connected time in one session.
///
/// Manager-gated: the summary exposes another actor's activity, which is
/// roster-level information. A session the caller cannot see is a `404`.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/participation",
    tag = "participation",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Per-participant connected time", body = SessionParticipationResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
    ),
)]
pub(crate) async fn session_summary(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionParticipationResponse>, ApiError> {
    let summary = session_participation(
        state.participation.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        SessionParticipation {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(summary.into()))
}
