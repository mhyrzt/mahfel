use super::api_key;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{ServicePrincipalStore, StoreError};
use mahfel_domain::{ActorId, ServicePrincipal};
use thiserror::Error;

/// Command to rotate the API key of an existing service principal.
#[derive(Debug, Clone)]
pub struct RotateServicePrincipalKey {
    pub id: ActorId,
}

/// The rotated principal together with its new raw API key.
///
/// `Debug` is intentionally not derived so the key cannot be logged by
/// accident.
#[derive(Clone)]
pub struct RotatedServicePrincipalKey {
    pub principal: ServicePrincipal,
    pub api_key: String,
}

#[derive(Debug, Error)]
pub enum RotateServicePrincipalKeyError {
    #[error("service principal not found")]
    NotFound,
    #[error("service principal is revoked")]
    Revoked,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for RotateServicePrincipalKeyError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Revoked => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Revoke the principal's active keys and issue a new one.
///
/// Rotation is refused for a revoked principal so a dead principal cannot be
/// revived through the rotation path.
pub async fn rotate_service_principal_key<S>(
    store: &S,
    command: RotateServicePrincipalKey,
) -> Result<RotatedServicePrincipalKey, RotateServicePrincipalKeyError>
where
    S: ServicePrincipalStore,
{
    let principal = store
        .get(command.id)
        .await?
        .ok_or(RotateServicePrincipalKeyError::NotFound)?;
    if !principal.is_active() {
        return Err(RotateServicePrincipalKeyError::Revoked);
    }
    let raw_key = api_key::generate();
    store
        .rotate_key(command.id, &api_key::hash(&raw_key))
        .await?;
    Ok(RotatedServicePrincipalKey {
        principal,
        api_key: raw_key,
    })
}
