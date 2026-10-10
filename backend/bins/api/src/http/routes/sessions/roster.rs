use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State, http::header, response::IntoResponse};
use futures_util::{StreamExt, stream};
use mahfel_application::sessions::get_session_for_roster;
use mahfel_domain::{FileContext, ParticipantStatus, RoomRole, SessionId};
use serde::Serialize;
use std::{collections::HashMap, time::Duration};
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
    let session = get_session_for_roster(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        state.realtime.stores.participants.as_ref(),
        state.realtime.stores.blocks.as_ref(),
        &principal,
        session_id,
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
    let active: Vec<_> = participants
        .into_iter()
        .filter(|participant| participant.status == ParticipantStatus::Active)
        .collect();
    let actors: Vec<_> = active
        .iter()
        .map(|participant| participant.actor_id)
        .collect();
    let users: HashMap<_, _> = state
        .users
        .get_many(&actors)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "roster user lookup failed");
            ApiError::internal("session roster unavailable")
        })?
        .into_iter()
        .map(|user| (user.id, user))
        .collect();
    let members: HashMap<_, _> = state
        .rooms
        .list_members(session.room_id)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "roster member lookup failed");
            ApiError::internal("session roster unavailable")
        })?
        .into_iter()
        .map(|member| (member.actor_id, member.role))
        .collect();
    let avatars: HashMap<_, _> = match state.files.list_avatars(&actors).await {
        Ok(files) => files
            .into_iter()
            .filter_map(|file| match file.context {
                FileContext::Avatar(actor) => Some((actor, file)),
                _ => None,
            })
            .collect(),
        Err(error) => {
            tracing::warn!(%error, "roster avatar lookup failed");
            HashMap::new()
        }
    };
    let roster: Vec<_> = stream::iter(active)
        .map(|participant| {
            let users = &users;
            let members = &members;
            let avatars = &avatars;
            let storage = &state.storage;
            async move {
                let user = users.get(&participant.actor_id);
                let name = user
                    .map(|user| user.display_name.as_str().to_owned())
                    .unwrap_or_else(|| format!("Participant {}", &participant.id.to_string()[..4]));
                let role = if members.get(&participant.actor_id) == Some(&RoomRole::Manager) {
                    "manager"
                } else {
                    "member"
                };
                let signed_avatar = if let Some(file) = avatars.get(&participant.actor_id) {
                    match storage
                        .presign_download(
                            &file.object_key,
                            &file.file_name,
                            Duration::from_secs(3600),
                        )
                        .await
                    {
                        Ok(signed) => Some(signed.url),
                        Err(error) => {
                            tracing::warn!(%error, "roster avatar signing failed");
                            None
                        }
                    }
                } else {
                    None
                };
                let avatar_url = signed_avatar.or_else(|| {
                    user.and_then(|user| {
                        user.avatar_url.as_ref().map(|url| url.as_str().to_owned())
                    })
                });
                RosterParticipantResponse {
                    id: participant.id.to_string(),
                    actor_id: participant.actor_id.to_string(),
                    name,
                    role,
                    avatar_url,
                }
            }
        })
        .buffered(8)
        .collect()
        .await;
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(roster)))
}
