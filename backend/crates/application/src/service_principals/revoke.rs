use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{ServicePrincipalStore, StoreError};
use mahfel_domain::ActorId;
use thiserror::Error;
use time::OffsetDateTime;

/// Command to revoke a service principal.
#[derive(Debug, Clone)]
pub struct RevokeServicePrincipal {
    pub id: ActorId,
}

#[derive(Debug, Error)]
pub enum RevokeServicePrincipalError {
    #[error("service principal not found")]
    NotFound,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for RevokeServicePrincipalError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Revoke a service principal and all of its active keys.
///
/// Revocation is terminal and rejects every subsequent request made with a key
/// that belonged to the principal. Revoking an already revoked principal is a
/// no-op so the operation is idempotent.
pub async fn revoke_service_principal<S>(
    store: &S,
    command: RevokeServicePrincipal,
) -> Result<(), RevokeServicePrincipalError>
where
    S: ServicePrincipalStore,
{
    let mut principal = store
        .get(command.id)
        .await?
        .ok_or(RevokeServicePrincipalError::NotFound)?;
    if principal.revoke(OffsetDateTime::now_utc()).is_err() {
        // Already revoked: the durable state already rejects requests.
        return Ok(());
    }
    store.revoke(command.id).await?;
    Ok(())
}
