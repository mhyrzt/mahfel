use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{FileStorageError, StoreError};
use mahfel_domain::FileError;

#[derive(Debug, thiserror::Error)]
pub enum FileOperationError {
    #[error("file not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("file is not ready")]
    NotReady,
    #[error("file is pinned by a pending recording archive")]
    ArchivePending,
    #[error("file is embedded in a collaboration document and cannot be deleted")]
    InUse,
    #[error("file contents failed verification")]
    Rejected,
    #[error("file verification unavailable")]
    VerificationUnavailable,
    #[error("invalid file: {0}")]
    Invalid(#[from] FileError),
    #[error("file metadata unavailable")]
    Store(#[from] StoreError),
    #[error("object storage unavailable")]
    Storage(#[from] FileStorageError),
}

impl ApplicationError for FileOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound
            | Self::Store(StoreError::MissingReference(_))
            | Self::Storage(FileStorageError::NotFound) => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::NotReady
            | Self::ArchivePending
            | Self::InUse
            | Self::Store(StoreError::Conflict(_)) => ErrorKind::Conflict,
            Self::Rejected | Self::Invalid(_) | Self::Storage(FileStorageError::TooLarge) => {
                ErrorKind::Unprocessable
            }
            Self::VerificationUnavailable | Self::Store(_) => ErrorKind::Internal,
            Self::Storage(FileStorageError::Unavailable) => ErrorKind::Unavailable,
        }
    }
}
