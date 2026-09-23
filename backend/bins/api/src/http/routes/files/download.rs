use super::dto::{SignedFileResponse, SignedHeader};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, Router, extract::State, http::header, response::IntoResponse, routing::get};
use mahfel_application::files::{DownloadFile, download_file};
use uuid::Uuid;

pub(crate) fn route() -> Router<AppState> {
    Router::new().route("/files/{file_id}/download", get(download))
}

#[utoipa::path(
    get,
    path = "/v1/files/{file_id}/download",
    tag = "files",
    summary = "Create a direct download authorization",
    description = "Checks current access to the file context, records the authorization, and returns a short-lived S3 URL with private no-store caching.",
    operation_id = "downloadFile",
    params(
        ("file_id" = Uuid, Path, description = "File identifier returned by upload initiation")
    ),
    responses(
        (status = 200, description = "Short-lived signed GET and required headers", body = SignedFileResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "File not found or inaccessible")
    )
)]
pub(crate) async fn download(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(id): ValidatedPath<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let (file, signed) = download_file(
        state.files.as_ref(),
        state.rooms.as_ref(),
        state.storage.as_ref(),
        DownloadFile {
            actor: principal.actor_id,
            id: mahfel_domain::FileId::from_uuid(id),
        },
    )
    .await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(SignedFileResponse {
            file: file.into(),
            url: signed.url,
            headers: signed
                .headers
                .into_iter()
                .map(|(name, value)| SignedHeader { name, value })
                .collect(),
            expires_in_seconds: 60,
        }),
    ))
}
