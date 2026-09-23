use super::dto::{FileContextInput, FileContextKind, FileListResponse, context};
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get},
};
use mahfel_application::files::{DeleteFile, ListFiles, delete_file, list_files};
use uuid::Uuid;

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/files", get(list))
        .route("/files/{file_id}", delete(remove))
}

#[utoipa::path(
    get,
    path = "/v1/files",
    tag = "files",
    summary = "List files in a context",
    description = "Returns ready files only. The caller must have read access to the selected context.",
    operation_id = "listFiles",
    params(
        ("context_kind" = FileContextKind, Query, description = "room, session, chat_message, avatar, or recording"),
        ("context_id" = Uuid, Query, description = "Identifier of the selected context"),
        ("limit" = Option<u32>, Query, description = "Page size from 1 to 100; defaults to 50"),
        ("offset" = Option<u64>, Query, description = "Number of ready files to skip")
    ),
    responses(
        (status = 200, description = "Ready files in the selected context", body = FileListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Context not found or inaccessible")
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Query(query): Query<FileContextInput>,
) -> Result<Json<FileListResponse>, ApiError> {
    let files = list_files(
        state.files.as_ref(),
        state.rooms.as_ref(),
        ListFiles {
            actor: principal.actor_id,
            context: context(query.context_kind, query.context_id),
            limit: query.limit.unwrap_or(50),
            offset: query.offset.unwrap_or(0),
        },
    )
    .await?;
    Ok(Json(FileListResponse {
        files: files.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(
    delete,
    path = "/v1/files/{file_id}",
    tag = "files",
    summary = "Delete a file",
    description = "Removes the file metadata and its staged or published storage objects. Owners and context managers may delete it.",
    operation_id = "deleteFile",
    params(
        ("file_id" = Uuid, Path, description = "File identifier returned by upload initiation")
    ),
    responses(
        (status = 204, description = "File and its storage objects removed"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "File not found or caller cannot delete it")
    )
)]
pub(crate) async fn remove(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(id): ValidatedPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    delete_file(
        state.files.as_ref(),
        state.rooms.as_ref(),
        state.storage.as_ref(),
        DeleteFile {
            actor: principal.actor_id,
            id: mahfel_domain::FileId::from_uuid(id),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
