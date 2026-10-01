use crate::http::{
    auth::Authenticated,
    error::{ApiError, api_error_from},
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
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
        .route(
            "/sessions/{session_id}/recordings/{recording_id}/export",
            get(export_status).post(export_retry),
        )
        .route(
            "/sessions/{session_id}/recordings/{recording_id}/export/download",
            get(export_download),
        )
        .route(
            "/sessions/{session_id}/recordings/{recording_id}/playback",
            get(playback),
        )
        .route(
            "/sessions/{session_id}/recordings/{recording_id}/playback/media-window",
            get(playback_media_window),
        )
        .route(
            "/sessions/{session_id}/recordings/{recording_id}/playback/retry",
            post(playback_retry),
        )
        .route(
            "/sessions/{session_id}/recordings/{recording_id}",
            axum::routing::delete(remove),
        )
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

#[derive(Serialize, ToSchema)]
pub(crate) struct RecordingExportResponse {
    pub id: Uuid,
    pub recording_id: Uuid,
    pub status: String,
    pub progress: u16,
    pub duration_ms: Option<i64>,
    pub size_bytes: Option<i64>,
    pub failure_code: Option<String>,
    pub partial: bool,
}
impl From<mahfel_domain::RecordingExport> for RecordingExportResponse {
    fn from(export: mahfel_domain::RecordingExport) -> Self {
        use mahfel_domain::RecordingExportStatus;
        Self {
            id: export.id,
            recording_id: export.recording_id,
            status: match export.status {
                RecordingExportStatus::Queued => "queued",
                RecordingExportStatus::Rendering => "rendering",
                RecordingExportStatus::Ready => "ready",
                RecordingExportStatus::Failed => "failed",
            }
            .into(),
            progress: export.progress,
            duration_ms: export.duration_ms,
            size_bytes: export.size_bytes,
            failure_code: export.failure_code,
            partial: export.partial,
        }
    }
}
#[derive(Serialize, ToSchema)]
pub(crate) struct RecordingDownloadResponse {
    pub url: String,
}

#[utoipa::path(get, operation_id = "getRecordingExport", path = "/v1/sessions/{session_id}/recordings/{recording_id}/export", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 200, body = RecordingExportResponse), (status = 401), (status = 404), (status = 503)))]
pub(crate) async fn export_status(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<RecordingExportResponse>, ApiError> {
    let store = state.recording_exports.as_deref().ok_or_else(|| {
        ApiError::from_application(
            ErrorKind::Unavailable,
            "recording export unavailable".into(),
        )
    })?;
    Ok(Json(
        mahfel_application::recordings::get_export(
            store,
            SessionId::from_uuid(session),
            recording,
            &principal,
        )
        .await?
        .into(),
    ))
}

#[utoipa::path(post, operation_id = "retryRecordingExport", path = "/v1/sessions/{session_id}/recordings/{recording_id}/export", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 200, body = RecordingExportResponse), (status = 401), (status = 403), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn export_retry(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<RecordingExportResponse>, ApiError> {
    let store = state.recording_exports.as_deref().ok_or_else(|| {
        ApiError::from_application(
            ErrorKind::Unavailable,
            "recording export unavailable".into(),
        )
    })?;
    Ok(Json(
        mahfel_application::recordings::retry_export(
            store,
            state.rooms.as_ref(),
            state.sessions.as_ref(),
            SessionId::from_uuid(session),
            recording,
            &principal,
        )
        .await?
        .into(),
    ))
}

#[utoipa::path(get, operation_id = "downloadRecordingExport", path = "/v1/sessions/{session_id}/recordings/{recording_id}/export/download", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 200, body = RecordingDownloadResponse), (status = 401), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn export_download(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<RecordingDownloadResponse>, ApiError> {
    let store = state.recording_exports.as_deref().ok_or_else(|| {
        ApiError::from_application(
            ErrorKind::Unavailable,
            "recording export unavailable".into(),
        )
    })?;
    let export = mahfel_application::recordings::get_export(
        store,
        SessionId::from_uuid(session),
        recording,
        &principal,
    )
    .await?;
    if export.status != mahfel_domain::RecordingExportStatus::Ready {
        return Err(ApiError::from_application(
            ErrorKind::Conflict,
            "recording export is not ready".into(),
        ));
    }
    let key = export.object_key.ok_or_else(|| {
        ApiError::from_application(ErrorKind::Internal, "missing export artifact".into())
    })?;
    let signed = state
        .storage
        .presign_download(
            &key,
            &format!("recording-{recording}.mp4"),
            std::time::Duration::from_secs(300),
        )
        .await
        .map_err(|_| {
            ApiError::from_application(
                ErrorKind::Unavailable,
                "recording storage unavailable".into(),
            )
        })?;
    Ok(Json(RecordingDownloadResponse { url: signed.url }))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct RecordingPlaybackResponse {
    pub manifest_key: String,
    pub urls: std::collections::BTreeMap<String, String>,
    pub assets: std::collections::BTreeMap<String, String>,
    pub media: Vec<PlaybackMediaSegmentResponse>,
    pub expires_in_seconds: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PlaybackMediaSegmentResponse {
    pub track_id: String,
    pub actor_id: String,
    pub source: String,
    pub audio: bool,
    pub start_ns: String,
    pub end_ns: String,
    pub object_key: String,
}

impl From<mahfel_application::recordings::PlaybackMediaSegment> for PlaybackMediaSegmentResponse {
    fn from(segment: mahfel_application::recordings::PlaybackMediaSegment) -> Self {
        Self {
            track_id: segment.track_id,
            actor_id: segment.actor_id,
            source: segment.source,
            audio: segment.audio,
            start_ns: segment.start_ns,
            end_ns: segment.end_ns,
            object_key: segment.object_key,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct PlaybackWindowQuery {
    start_ms: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PlaybackWindowResponse {
    urls: std::collections::BTreeMap<String, String>,
}

#[utoipa::path(get, operation_id = "getRecordingPlaybackMediaWindow", path = "/v1/sessions/{session_id}/recordings/{recording_id}/playback/media-window", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path), ("start_ms" = u64, Query)), responses((status = 200, body = PlaybackWindowResponse), (status = 401), (status = 404), (status = 503)))]
pub(crate) async fn playback_media_window(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
    Query(query): Query<PlaybackWindowQuery>,
) -> Result<Json<PlaybackWindowResponse>, ApiError> {
    let store = state.recording_playback.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".into())
    })?;
    let window = mahfel_application::recordings::playback_window(
        store,
        state.storage.as_ref(),
        SessionId::from_uuid(session),
        recording,
        &principal,
        query.start_ms,
        std::time::Duration::from_secs(PLAYBACK_URL_TTL_SECONDS),
    )
    .await?;
    Ok(Json(PlaybackWindowResponse { urls: window.urls }))
}

#[utoipa::path(post, operation_id = "retryRecordingPlayback", path = "/v1/sessions/{session_id}/recordings/{recording_id}/playback/retry", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 202), (status = 401), (status = 403), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn playback_retry(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let store = state.recording_playback.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".into())
    })?;
    mahfel_application::recordings::retry_playback(
        store,
        state.rooms.as_ref(),
        state.sessions.as_ref(),
        SessionId::from_uuid(session),
        recording,
        &principal,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

const PLAYBACK_URL_TTL_SECONDS: u64 = 300;

#[utoipa::path(get, operation_id = "getRecordingPlayback", path = "/v1/sessions/{session_id}/recordings/{recording_id}/playback", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 200, body = RecordingPlaybackResponse), (status = 401), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn playback(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<RecordingPlaybackResponse>, ApiError> {
    let store = state.recording_playback.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".into())
    })?;
    let descriptor = mahfel_application::recordings::playback_descriptor(
        store,
        state.storage.as_ref(),
        SessionId::from_uuid(session),
        recording,
        &principal,
        std::time::Duration::from_secs(PLAYBACK_URL_TTL_SECONDS),
    )
    .await?;
    Ok(Json(RecordingPlaybackResponse {
        manifest_key: descriptor.manifest_key,
        urls: descriptor.urls,
        assets: descriptor.assets,
        media: descriptor.media.into_iter().map(Into::into).collect(),
        expires_in_seconds: PLAYBACK_URL_TTL_SECONDS,
    }))
}

#[utoipa::path(delete, operation_id = "deleteRecording", path = "/v1/sessions/{session_id}/recordings/{recording_id}", tag = "recordings", params(("session_id" = Uuid, Path), ("recording_id" = Uuid, Path)), responses((status = 204), (status = 401), (status = 403), (status = 404), (status = 409), (status = 503)))]
pub(crate) async fn remove(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session, recording)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let store = state.recording_playback.as_deref().ok_or_else(|| {
        ApiError::from_application(ErrorKind::Unavailable, "recording unavailable".into())
    })?;
    mahfel_application::recordings::delete_recording(
        store,
        state.rooms.as_ref(),
        state.sessions.as_ref(),
        SessionId::from_uuid(session),
        recording,
        &principal,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
