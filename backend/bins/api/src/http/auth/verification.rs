use futures_util::future::BoxFuture;
use mahfel_auth::{AuthenticatedIdentity, TokenError, TokenVerifier};
use std::sync::Arc;
use thiserror::Error;

/// Why token-based authentication failed.
///
/// The cause is preserved for logs; it is never sent to a client.
#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("token is missing or malformed")]
    MissingToken,
    #[error("token was rejected: {0}")]
    Token(#[from] TokenError),
    /// The token is valid but no application user has been linked yet.
    #[error("account is not provisioned")]
    NotProvisioned,
    /// A service API key was presented but is unknown, revoked, or inactive.
    #[error("credential is invalid")]
    InvalidCredential,
    /// Authentication could not be completed because storage failed.
    #[error("authentication backend is unavailable")]
    Internal,
}

/// Verifies a bearer token and returns the provider-neutral claims.
///
/// Dyn-compatible so the same implementation serves HTTP extraction and the
/// WebSocket upgrade, and so tests can substitute a stub.
pub trait RequestAuthenticator: Send + Sync + 'static {
    fn verify<'a>(
        &'a self,
        token: Option<&'a str>,
    ) -> BoxFuture<'a, Result<AuthenticatedIdentity, AuthenticationError>>;
}

/// Verifies Keycloak tokens.
#[derive(Clone)]
pub struct TokenAuthentication {
    verifier: Arc<TokenVerifier>,
}

impl TokenAuthentication {
    #[must_use]
    pub fn new(verifier: TokenVerifier) -> Self {
        Self {
            verifier: Arc::new(verifier),
        }
    }
}

impl RequestAuthenticator for TokenAuthentication {
    fn verify<'a>(
        &'a self,
        token: Option<&'a str>,
    ) -> BoxFuture<'a, Result<AuthenticatedIdentity, AuthenticationError>> {
        Box::pin(async move {
            let token = token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or(AuthenticationError::MissingToken)?;
            self.verifier.verify(token).await.map_err(Into::into)
        })
    }
}
