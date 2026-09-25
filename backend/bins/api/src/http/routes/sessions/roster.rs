use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State, http::header, response::IntoResponse};
use mahfel_application::sessions::{GetSessionForCaller, get_session_for_caller};
use mahfel_domain::{ParticipantStatus, RoomRole, SessionId};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub(crate) struct RosterParticipantResponse {
    id: String,
    actor_id: String,
    name: String,
    role: &'static str,
    avatar_url: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/roster",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Active session participants", body = Vec<RosterParticipantResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session does not exist or the caller cannot see it"),
    ),
)]
pub(crate) async fn roster(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
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
    let participants = state
        .realtime
        .stores
        .participants
        .list_for_session(session_id)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "session roster lookup failed");
            ApiError::internal("session roster unavailable")
        })?;
    let mut roster = Vec::new();
    for participant in participants
        .into_iter()
        .filter(|participant| participant.status == ParticipantStatus::Active)
    {
        let user = state
            .users
            .get(participant.actor_id)
            .await
            .map_err(|error| {
                tracing::warn!(%error, "roster user lookup failed");
                ApiError::internal("session roster unavailable")
            })?;
        let name = user
            .as_ref()
            .map(|user| user.display_name.as_str().to_owned())
            .unwrap_or_else(|| format!("Participant {}", &participant.id.to_string()[..4]));
        let member = state
            .rooms
            .get_member(session.room_id, participant.actor_id)
            .await
            .map_err(|error| {
                tracing::warn!(%error, "roster member lookup failed");
                ApiError::internal("session roster unavailable")
            })?;
        let role = if member.is_some_and(|member| member.role == RoomRole::Manager) {
            "manager"
        } else {
            "member"
        };
        let avatar_url = super::super::avatar::ready_avatar_url(&state, participant.actor_id)
            .await
            .or_else(|| user.and_then(|user| user.avatar_url.map(|url| url.as_str().to_owned())));
        roster.push(RosterParticipantResponse {
            id: participant.id.to_string(),
            actor_id: participant.actor_id.to_string(),
            name,
            role,
            avatar_url,
        });
    }
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(roster)))
}
