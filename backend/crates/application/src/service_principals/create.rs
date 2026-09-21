use super::api_key;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{ServicePrincipalStore, StoreError};
use mahfel_domain::{DisplayName, ServicePrincipal, ValidationError};
use thiserror::Error;
use time::OffsetDateTime;

/// Command to provision a new service principal.
#[derive(Debug, Clone)]
pub struct CreateServicePrincipal {
    /// Unique operator-facing name.
    pub name: String,
    /// Optional human-readable label.
    pub display_name: Option<String>,
}

/// A newly created principal together with its raw API key.
///
/// The raw key is present here and nowhere else; it is never re-readable.
/// `Debug` is intentionally not derived so the key cannot be logged by
/// accident.
#[derive(Clone)]
pub struct CreatedServicePrincipal {
    pub principal: ServicePrincipal,
    pub api_key: String,
}

#[derive(Debug, Error)]
pub enum CreateServicePrincipalError {
    #[error("invalid service principal name: {0}")]
    InvalidName(#[from] ValidationError),
    #[error("a service principal with that name already exists")]
    NameTaken,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for CreateServicePrincipalError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidName(_) => ErrorKind::Unprocessable,
            Self::NameTaken => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Provision a service principal and issue its first API key.
pub async fn create_service_principal<S>(
    store: &S,
    command: CreateServicePrincipal,
) -> Result<CreatedServicePrincipal, CreateServicePrincipalError>
where
    S: ServicePrincipalStore,
{
    let name = DisplayName::new(command.name)?;
    let display_name = command
        .display_name
        .map(DisplayName::new)
        .transpose()
        .map_err(CreateServicePrincipalError::InvalidName)?;
    let principal = ServicePrincipal::create(name, display_name, OffsetDateTime::now_utc());
    let raw_key = api_key::generate();
    match store.create(&principal, &api_key::hash(&raw_key)).await {
        Ok(()) => Ok(CreatedServicePrincipal {
            principal,
            api_key: raw_key,
        }),
        Err(StoreError::Conflict(_)) => Err(CreateServicePrincipalError::NameTaken),
        Err(error) => Err(error.into()),
    }
}
