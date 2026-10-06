use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
    ws::ServerMessage,
};
use axum::{extract::State, http::StatusCode};
use mahfel_application::ports::CoordinationStore;
use mahfel_domain::SessionId;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub(crate) struct ReactionRequest {
    pub emoji: String,
}

fn allowed(emoji: &str) -> bool {
    matches!(emoji, "👍" | "👎" | "👏" | "❤️" | "😂" | "😮" | "🎉" | "🔥")
}

#[utoipa::path(post, path = "/v1/sessions/{session_id}/interactions/reactions", tag = "interactions", params(("session_id" = String, Path)), request_body = ReactionRequest, responses((status = 204), (status = 403), (status = 404), (status = 422), (status = 429), (status = 503)))]
pub(crate) async fn react(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session): ValidatedPath<Uuid>,
    ValidatedJson(request): ValidatedJson<ReactionRequest>,
) -> Result<StatusCode, ApiError> {
    if !allowed(&request.emoji) {
        return Err(ApiError::unprocessable("unsupported reaction"));
    }
    let session = SessionId::from_uuid(session);
    let _guard = state
        .realtime
        .lock_session_actor(session, principal.actor_id)
        .await
        .map_err(|_| ApiError::unavailable("session presence unavailable"))?;
    let store = state
        .interactions
        .as_deref()
        .ok_or_else(|| ApiError::unavailable("session interactions unavailable"))?;
    if !store.can_respond(session, &principal).await? {
        return Err(ApiError::forbidden("join the session first"));
    }
    let participant = state
        .realtime
        .stores
        .participants
        .get_active(session, principal.actor_id)
        .await
        .map_err(|_| ApiError::unavailable("session presence unavailable"))?
        .ok_or_else(|| ApiError::forbidden("join the session first"))?;
    let redis = state
        .realtime
        .session_presence
        .as_ref()
        .ok_or_else(|| ApiError::unavailable("session reactions unavailable"))?;
    let presence = redis
        .session_presence_snapshot(session)
        .await
        .map_err(|_| ApiError::unavailable("session presence unavailable"))?;
    if !presence.participant_ids.contains(&participant.id) {
        return Err(ApiError::forbidden("join the session first"));
    }
    let actor_key = format!(
        "session-reaction-actor:{:x}",
        Sha256::digest(principal.actor_id.to_string())
    );
    let session_key = format!(
        "session-reaction-room:{:x}",
        Sha256::digest(session.to_string())
    );
    let window = Duration::from_secs(2);
    let actor_count = redis
        .increment_with_ttl(&actor_key, window)
        .await
        .map_err(|_| ApiError::unavailable("session reactions unavailable"))?;
    if actor_count > 2 {
        return Err(ApiError::from_application(
            mahfel_application::ErrorKind::RateLimited,
            "wait before reacting again".into(),
        ));
    }
    let session_count = redis
        .increment_with_ttl(&session_key, window)
        .await
        .map_err(|_| ApiError::unavailable("session reactions unavailable"))?;
    if session_count > 60 {
        return Err(ApiError::from_application(
            mahfel_application::ErrorKind::RateLimited,
            "too many session reactions".into(),
        ));
    }
    let username = principal
        .username
        .as_deref()
        .or(principal.display_name.as_deref())
        .unwrap_or("Participant")
        .chars()
        .take(80)
        .collect();
    let message = ServerMessage::SessionReaction {
        session_id: session.to_string(),
        event_id: Uuid::new_v4().to_string(),
        participant_id: participant.id.to_string(),
        username,
        emoji: request.emoji,
    };
    let payload = serde_json::to_vec(&message)
        .map_err(|_| ApiError::unavailable("session reactions unavailable"))?;
    redis
        .publish(&redis.keys().channel("session-interactions"), &payload)
        .await
        .map_err(|_| ApiError::unavailable("session reactions unavailable"))?;
    state
        .realtime
        .registry
        .broadcast_to_session(session, &message);
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reactions_accept_only_the_bounded_palette() {
        for emoji in ["👍", "👎", "👏", "❤️", "😂", "😮", "🎉", "🔥"] {
            assert!(super::allowed(emoji));
        }
        for input in ["", "👍👍", "<script>", "custom", "👍\n"] {
            assert!(!super::allowed(input));
        }
    }
}
