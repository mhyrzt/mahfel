use super::dto::{SessionResponse, response_with_room};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{StartSession, start_session_with_recording};
use mahfel_domain::SessionId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/start",
    summary = "Start a scheduled session",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Session is live", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller lacks manage_session"),
        (status = 404, description = "Session does not exist or the caller cannot see its room"),
        (status = 409, description = "Invalid transition or a live session already exists"),
        (status = 503, description = "Recording is unavailable; the session remains scheduled"),
    ),
)]
pub(crate) async fn start(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionResponse>, ApiError> {
    let realtime = state.realtime.clone();
    let session = start_session_with_recording(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        StartSession {
            principal,
            session_id: SessionId::from_uuid(session_id),
        },
        move |session, room| async move {
            let result = async {
                let media = match &realtime.placement {
                    Some(placement) => placement.place(session, room).await?,
                    None => realtime.media.clone(),
                };
                let media_room = if realtime.placement.is_some() {
                    crate::http::ws::media_room_id(session)
                } else {
                    room
                };
                media.prepare_recording(media_room.to_string()).await?;
                Ok::<_, anyhow::Error>(())
            }.await;
            if let Err(error) = result {
                tracing::warn!(%error, session_id = %session, "recorded session start readiness failed");
                false
            } else {
                true
            }
        },
    )
    .await?;
    Ok(Json(response_with_room(&state, session).await?))
}
