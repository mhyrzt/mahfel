use crate::http::{
    auth::Authenticated,
    error::{ApiError, api_error_from},
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use mahfel_application::{
    ErrorKind,
    recordings::{RecordingOperationError, control_recording, list_recordings},
};
use mahfel_domain::{Recording, RecordingAction, SessionId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

api_error_from!(RecordingOperationError);

#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecordingActionDto {
    Start,
    Pause,
    Resume,
    Stop,
}

impl From<RecordingActionDto> for RecordingAction {
    fn from(action: RecordingActionDto) -> Self {
        match action {
            RecordingActionDto::Start => Self::Start,
            RecordingActionDto::Pause => Self::Pause,
            RecordingActionDto::Resume => Self::Resume,
            RecordingActionDto::Stop => Self::Stop,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct RecordingCommandRequest {
    pub command_id: Uuid,
    pub action: RecordingActionDto,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct RecordingResponse {
    pub id: String,
    pub session_id: String,
    pub status: String,
    pub revision: i64,
    pub capture_generation: i32,
    #[serde(with = "super::rfc3339")]
    pub started_at: OffsetDateTime,
    #[serde(with = "super::rfc3339::option")]
    pub stopped_at: Option<OffsetDateTime>,
    pub warning: Option<String>,
    pub partial: bool,
}

impl From<Recording> for RecordingResponse {
    fn from(recording: Recording) -> Self {
        Self {
            id: recording.id.to_string(),
            session_id: recording.session_id.to_string(),
            status: recording.status.as_str().to_owned(),
            revision: recording.revision,
            capture_generation: recording.capture_generation,
            started_at: recording.started_at,
            stopped_at: recording.stopped_at,
            warning: recording.warning,
            partial: recording.partial,
        }
    }
}

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}/recordings", get(list))
        .route("/sessions/{session_id}/recordings/commands", post(command))
}

#[utoipa::path(get, operation_id = "listSessionRecordings", path = "/v1/sessions/{session_id}/recordings", tag = "recordings",
    params(("session_id" = Uuid, Path)),
    responses((status = 200, body = Vec<RecordingResponse>), (status = 401), (status = 404), (status = 503)))]
pub(crate) async fn list(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<Vec<RecordingResponse>>, ApiError> {
    let store = state.recordings.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".to_owned())
    })?;
    let recordings = list_recordings(store, SessionId::from_uuid(session_id), &principal).await?;
    Ok(Json(recordings.into_iter().map(Into::into).collect()))
}

#[utoipa::path(post, operation_id = "controlSessionRecording", path = "/v1/sessions/{session_id}/recordings/commands", tag = "recordings",
    params(("session_id" = Uuid, Path)), request_body = RecordingCommandRequest,
    responses((status = 200, body = RecordingResponse), (status = 401), (status = 403), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn command(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<RecordingCommandRequest>,
) -> Result<Json<RecordingResponse>, ApiError> {
    let store = state.recordings.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".to_owned())
    })?;
    let recording = control_recording(
        store,
        state.rooms.as_ref(),
        state.sessions.as_ref(),
        SessionId::from_uuid(session_id),
        &principal,
        body.command_id,
        body.action.into(),
    )
    .await?;
    Ok(Json(recording.into()))
}
