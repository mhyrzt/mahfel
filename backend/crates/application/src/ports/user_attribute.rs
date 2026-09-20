use crate::error::{ApplicationError, ErrorKind};
use std::{future::Future, pin::Pin};
use thiserror::Error;

/// Failure to write a provider-side identity attribute.
#[derive(Debug, Error)]
pub enum AttributeError {
    /// The provider is unreachable or timed out. Callers should degrade.
    #[error("identity provider unavailable: {0}")]
    Unavailable(String),
    /// The provider rejected the operation. Messages must not reach clients.
    #[error("identity provider failure: {0}")]
    Unexpected(String),
}

impl ApplicationError for AttributeError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Unavailable(_) => ErrorKind::Unavailable,
            Self::Unexpected(_) => ErrorKind::Internal,
        }
    }
}

/// Writes provider-side attributes onto an identity.
///
/// Used to link the local application user id onto the provider identity so a
/// protocol mapper can surface it as a token claim. Boxed futures keep the port
/// dyn-compatible, because transports hold one adapter behind `Arc<dyn ...>`.
pub trait UserAttributeWriter: Send + Sync {
    fn set_user_attribute<'a>(
        &'a self,
        user_id: &'a str,
        name: &'a str,
        value: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AttributeError>> + Send + 'a>>;
}
