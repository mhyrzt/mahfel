pub(super) mod download;
mod dto;
pub(super) mod library;
pub(super) mod upload;

use crate::http::{error::api_error_from, state::AppState};
use axum::Router;
use mahfel_application::files::FileOperationError;

api_error_from!(FileOperationError);

pub(crate) use dto::{
    FileContextInput, FileContextKind, FileListResponse, FileResponse, InitiateUploadRequest,
    SignedFileResponse, SignedHeader,
};
pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .merge(upload::route())
        .merge(download::route())
        .merge(library::route())
}
