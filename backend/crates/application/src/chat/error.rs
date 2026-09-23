use crate::{ApplicationError, ErrorKind, ports::StoreError};
use mahfel_domain::ChatMessageError;

#[derive(Debug, thiserror::Error)]
pub enum ChatOperationError {
    #[error(transparent)]
    Message(#[from] ChatMessageError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ApplicationError for ChatOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Message(_) => ErrorKind::Unprocessable,
            Self::Store(StoreError::NotFound(_) | StoreError::MissingReference(_)) => {
                ErrorKind::NotFound
            }
            Self::Store(StoreError::Conflict(_)) => ErrorKind::Conflict,
            Self::Store(StoreError::Unexpected(_)) => ErrorKind::Internal,
        }
    }
}
