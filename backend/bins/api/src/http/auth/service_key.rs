//! Service principal authentication through Mahfel-issued API keys.
//!
//! A service principal authenticates with an opaque `mhf_`-prefixed bearer key
//! (ADR 0022). The raw key is hashed with SHA-256 and matched against active key
//! rows; a match yields a [`Principal`] whose kind is `service`. The raw key is
//! never logged or persisted.

use super::verification::AuthenticationError;
use futures_util::future::BoxFuture;
use mahfel_application::ports::{Principal, ServicePrincipalStore};
use mahfel_application::service_principals::{api_key, authenticate_service_principal};
use mahfel_domain::{ActorKind, ServicePrincipal};

/// Whether a bearer credential is a Mahfel service API key rather than a JWT.
#[must_use]
pub fn is_service_key(credential: &str) -> bool {
    credential.starts_with(api_key::KEY_PREFIX)
}

/// Authenticates a raw service API key into a principal.
///
/// Dyn-compatible so the HTTP extractor can hold it behind an `Arc`.
pub trait ServiceKeyAuthenticator: Send + Sync + 'static {
    fn authenticate<'a>(
        &'a self,
        raw_key: &'a str,
    ) -> BoxFuture<'a, Result<Principal, AuthenticationError>>;
}

/// [`ServiceKeyAuthenticator`] backed by a [`ServicePrincipalStore`].
#[derive(Clone)]
pub struct StoreServiceKeyAuthenticator<S> {
    store: S,
}

impl<S> StoreServiceKeyAuthenticator<S> {
    #[must_use]
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> ServiceKeyAuthenticator for StoreServiceKeyAuthenticator<S>
where
    S: ServicePrincipalStore + 'static,
{
    fn authenticate<'a>(
        &'a self,
        raw_key: &'a str,
    ) -> BoxFuture<'a, Result<Principal, AuthenticationError>> {
        Box::pin(async move {
            match authenticate_service_principal(&self.store, raw_key).await {
                Ok(Some(principal)) => Ok(principal_from_service(principal)),
                Ok(None) => Err(AuthenticationError::InvalidCredential),
                Err(_) => Err(AuthenticationError::Internal),
            }
        })
    }
}

fn principal_from_service(principal: ServicePrincipal) -> Principal {
    Principal {
        display_name: Some(principal.display_name().to_string()),
        actor_id: principal.id,
        kind: ActorKind::Service,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_prefixed_credentials_take_the_api_key_path() {
        assert!(is_service_key("mhf_abc"));
        assert!(!is_service_key("eyJhbGciOiJSUzI1NiJ9.token"));
        assert!(!is_service_key(""));
    }
}
