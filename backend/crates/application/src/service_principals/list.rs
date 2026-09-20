use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{ServicePrincipalStore, StoreError};
use mahfel_domain::ServicePrincipal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListServicePrincipalsError {
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for ListServicePrincipalsError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// List every service principal, active or revoked, newest first.
pub async fn list_service_principals<S>(
    store: &S,
) -> Result<Vec<ServicePrincipal>, ListServicePrincipalsError>
where
    S: ServicePrincipalStore,
{
    Ok(store.list().await?)
}
