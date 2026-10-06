use super::dto::{InteractionRequest, InteractionsResponse, VoterResponse, VotersResponse};
use crate::http::{
    auth::Authenticated,
    error::{ApiError, api_error_from},
    extractor::{ValidatedJson, ValidatedPath, ValidatedQuery},
    state::AppState,
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use mahfel_application::{
    interactions,
    ports::{InteractionError, InteractionStore},
};
use mahfel_domain::SessionId;
use serde::Deserialize;
use uuid::Uuid;

api_error_from!(InteractionError);
fn store(state: &AppState) -> Result<&dyn InteractionStore, ApiError> {
    state
        .interactions
        .as_deref()
        .ok_or_else(|| ApiError::unavailable("session interactions unavailable"))
}
#[utoipa::path(get, path = "/v1/sessions/{session_id}/interactions", tag = "interactions", params(("session_id" = String, Path)), responses((status = 200, body = InteractionsResponse), (status = 404)))]
pub(crate) async fn snapshot(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session): ValidatedPath<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let snapshot = store(&state)?
        .snapshot(SessionId::from_uuid(session), &principal)
        .await?;
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(InteractionsResponse::new(snapshot, principal.actor_id)),
    ))
}
#[utoipa::path(post, path = "/v1/sessions/{session_id}/interactions/commands", tag = "interactions", params(("session_id" = String, Path)), request_body = InteractionRequest, responses((status = 204), (status = 403), (status = 404), (status = 409), (status = 422)))]
pub(crate) async fn command(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session): ValidatedPath<Uuid>,
    ValidatedJson(request): ValidatedJson<InteractionRequest>,
) -> Result<StatusCode, ApiError> {
    let session = SessionId::from_uuid(session);
    let is_hand = matches!(
        request.command,
        super::dto::InteractionAction::SetHand { .. }
            | super::dto::InteractionAction::Vote { .. }
            | super::dto::InteractionAction::Retract { .. }
    );
    let _guard = if is_hand {
        state
            .realtime
            .lock_session_actor(session, principal.actor_id)
            .await
            .map_err(|_| ApiError::unavailable("session presence unavailable"))?
    } else {
        None
    };
    if is_hand {
        let participant = state
            .realtime
            .stores
            .participants
            .get_active(session, principal.actor_id)
            .await
            .map_err(|_| ApiError::unavailable("session presence unavailable"))?
            .ok_or_else(|| ApiError::forbidden("join the session first"))?;
        if let Some(redis) = &state.realtime.session_presence {
            let presence = redis
                .session_presence_snapshot(session)
                .await
                .map_err(|_| ApiError::unavailable("session presence unavailable"))?;
            if !presence.participant_ids.contains(&participant.id) {
                return Err(ApiError::forbidden("join the session first"));
            }
        }
    }
    interactions::execute(
        store(&state)?,
        session,
        &principal,
        request.request_id,
        request.command.into(),
    )
    .await?;
    state.realtime.wake_interactions().await;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)]
pub(crate) struct VoterQuery {
    pub offset: Option<usize>,
}
#[utoipa::path(get, path = "/v1/sessions/{session_id}/interactions/activities/{activity_id}/voters", tag = "interactions", params(("session_id" = String, Path), ("activity_id" = String, Path), ("offset" = Option<usize>, Query)), responses((status = 200, body = VotersResponse), (status = 403), (status = 404)))]
pub(crate) async fn voters(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, activity)): ValidatedPath<(Uuid, Uuid)>,
    ValidatedQuery(query): ValidatedQuery<VoterQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let offset = query.offset.unwrap_or(0);
    if offset > 2000 {
        return Err(ApiError::bad_request("invalid voter offset"));
    }
    let snapshot = store(&state)?
        .snapshot(SessionId::from_uuid(session), &principal)
        .await?;
    let activity = snapshot
        .activities
        .into_iter()
        .find(|a| a.id == activity)
        .ok_or_else(|| ApiError::not_found("activity unavailable"))?;
    if activity.definition.anonymous
        || interactions::results(&activity, principal.actor_id).is_none()
    {
        return Err(ApiError::forbidden("voter names are not available"));
    }
    let votes: Vec<_> = activity
        .votes
        .into_iter()
        .filter(|v| !v.selected.is_empty())
        .collect();
    let next_offset = (offset + 100 < votes.len()).then_some(offset + 100);
    let voters = votes
        .into_iter()
        .skip(offset)
        .take(100)
        .map(|v| VoterResponse {
            name: v.name,
            actor_id: v.actor_id.to_string(),
            selected: v.selected,
        })
        .collect();
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(VotersResponse {
            voters,
            next_offset,
        }),
    ))
}
