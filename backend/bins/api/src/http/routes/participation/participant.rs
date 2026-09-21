use super::dto::{IntervalResponse, ParticipantParticipationResponse};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::participation::{ParticipantParticipation, participant_participation};
use mahfel_domain::{ParticipantId, SessionId};
use time::OffsetDateTime;
use uuid::Uuid;

/// One participant's connected time and interval history in one session.
///
/// Readable by the participant itself; a manager may read any participant in a
/// room they manage. Anyone else gets a `404`.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/participants/{participant_id}/participation",
    tag = "participation",
    params(
        ("session_id" = String, Path, description = "Session identifier (UUID)"),
        ("participant_id" = String, Path, description = "Participant identifier (UUID)"),
    ),
    responses(
        (status = 200, description = "Connected time and interval history", body = ParticipantParticipationResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session, participant record, or caller visibility not found"),
    ),
)]
pub(crate) async fn participant_summary(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session_id, participant_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<ParticipantParticipationResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let participant_id = ParticipantId::from_uuid(participant_id);
    let (intervals, total) = participant_participation(
        state.participation.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        ParticipantParticipation {
            actor: principal.actor_id,
            session_id,
            participant_id,
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    let now = OffsetDateTime::now_utc();
    Ok(Json(ParticipantParticipationResponse {
        session_id: session_id.to_string(),
        participant_id: participant_id.to_string(),
        connected_seconds: total,
        intervals: intervals
            .iter()
            .map(|interval| IntervalResponse::from_interval(interval, now))
            .collect(),
    }))
}
