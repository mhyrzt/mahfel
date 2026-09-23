use crate::{
    ApplicationError, ErrorKind,
    ports::{CoordinationError, StoreError},
};
use mahfel_domain::ChatMessageError;

#[derive(Debug, thiserror::Error)]
pub enum ChatOperationError {
    #[error("message page size must be between 1 and 100")]
    InvalidPageSize,
    #[error("chat posting limit exceeded")]
    RateLimited,
    #[error(transparent)]
    Coordination(#[from] CoordinationError),
    #[error(transparent)]
    Message(#[from] ChatMessageError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ApplicationError for ChatOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidPageSize => ErrorKind::InvalidInput,
            Self::RateLimited => ErrorKind::RateLimited,
            Self::Coordination(error) => error.kind(),
            Self::Message(_) => ErrorKind::Unprocessable,
            Self::Store(StoreError::NotFound(_) | StoreError::MissingReference(_)) => {
                ErrorKind::NotFound
            }
            Self::Store(StoreError::Conflict(_)) => ErrorKind::Conflict,
            Self::Store(StoreError::Unexpected(_)) => ErrorKind::Internal,
        }
    }
}
