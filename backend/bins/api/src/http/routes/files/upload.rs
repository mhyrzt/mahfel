use super::dto::{FileResponse, InitiateUploadRequest, SignedFileResponse, SignedHeader, context};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use mahfel_application::files::{CompleteUpload, InitiateUpload, complete_upload, initiate_upload};
use uuid::Uuid;

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/files/uploads", post(initiate))
        .route("/files/{file_id}/complete", post(complete))
}

#[utoipa::path(
    post,
    path = "/v1/files/uploads",
    tag = "files",
    summary = "Create a direct upload authorization",
    description = "Checks context access and metadata, then returns a short-lived S3 upload URL. Upload the bytes directly to storage and call the completion endpoint to verify them.",
    operation_id = "initiateFileUpload",
    request_body = InitiateUploadRequest,
    responses(
        (status = 201, description = "Short-lived signed PUT and required headers", body = SignedFileResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Context not found or inaccessible"),
        (status = 422, description = "Invalid file metadata"),
        (status = 503, description = "Storage unavailable")
    )
)]
pub(crate) async fn initiate(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedJson(body): ValidatedJson<InitiateUploadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let (file, signed) = initiate_upload(
        state.files.as_ref(),
        state.rooms.as_ref(),
        state.storage.as_ref(),
        InitiateUpload {
            actor: principal.actor_id,
            context: context(body.context_kind, body.context_id),
            file_name: body.file_name,
            mime: body.mime,
            size_bytes: body.size_bytes,
            checksum_sha256: body.checksum_sha256,
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(SignedFileResponse {
            file: file.into(),
            url: signed.url,
            headers: signed
                .headers
                .into_iter()
                .map(|(name, value)| SignedHeader { name, value })
                .collect(),
            expires_in_seconds: 300,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/v1/files/{file_id}/complete",
    tag = "files",
    summary = "Complete and verify an upload",
    description = "Reads the staged object, verifies its size, checksum, and content, then publishes it as a ready file. Repeated completion is safe for an already-ready file.",
    operation_id = "completeFileUpload",
    params(
        ("file_id" = Uuid, Path, description = "File identifier returned by upload initiation")
    ),
    responses(
        (status = 200, description = "Verified file metadata", body = FileResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "File not found or caller is not its owner"),
        (status = 409, description = "File is not in a completable state"),
        (status = 422, description = "Object failed verification")
    )
)]
pub(crate) async fn complete(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(id): ValidatedPath<Uuid>,
) -> Result<Json<FileResponse>, ApiError> {
    let file = complete_upload(
        state.files.as_ref(),
        state.storage.as_ref(),
        CompleteUpload {
            actor: principal.actor_id,
            id: mahfel_domain::FileId::from_uuid(id),
        },
    )
    .await?;
    Ok(Json(file.into()))
}
