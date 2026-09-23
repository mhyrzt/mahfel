use super::dto::{SessionResponse, response_with_room};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::ports::MediaGateway;
use mahfel_application::sessions::{EndSession, end_session};
use mahfel_domain::SessionId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/end",
    summary = "End a live session",
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
        state.participation.as_ref(),
        state.admissions.as_ref(),
        EndSession {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
        },
    )
    .await?;
    state
        .realtime
        .speakers
        .clear_session(session.id, &state.realtime.registry);
    match state.realtime.media_for(session.id, session.room_id).await {
        Ok(media) => {
            let media_room = if state.realtime.placement.is_some() {
                crate::http::ws::media_room_id(session.id)
            } else {
                session.room_id
            };
            if let Err(error) = MediaGateway::close_room(media.as_ref(), media_room).await {
                tracing::warn!(?error, room_id = %session.room_id, "could not release ended session room");
                if let Err(error) = media.clear_room_media(media_room).await {
                    tracing::warn!(?error, room_id = %session.room_id, "could not clear ended session media");
                }
            }
        }
        Err(error) => {
            tracing::warn!(?error, room_id = %session.room_id, "could not resolve ended session media node");
        }
    }
    Ok(Json(response_with_room(&state, session).await?))
}
