use mahfel_application::error::{ApplicationError, ErrorKind};
use thiserror::Error;

/// Keycloak Admin API failure.
///
/// Transport failures are `Unavailable` so callers can degrade; a rejected
/// service account is a configuration fault and stays internal.
#[derive(Debug, Error)]
pub enum KeycloakError {
    #[error("keycloak admin API unavailable: {0}")]
    Unavailable(String),
    #[error("keycloak rejected the admin service account: {0}")]
    Unauthorized(String),
    #[error("keycloak admin API failure: {0}")]
    Unexpected(String),
}

impl ApplicationError for KeycloakError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Unavailable(_) => ErrorKind::Unavailable,
            Self::Unauthorized(_) | Self::Unexpected(_) => ErrorKind::Internal,
        }
    }
}
