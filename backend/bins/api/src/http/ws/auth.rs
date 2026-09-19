use mahfel_application::ports::{AuthorizationError, Principal};

/// Close code used when a connection is not authenticated in time.
pub const CLOSE_UNAUTHORIZED: u16 = 4401;

/// Authenticate a WebSocket client.
///
/// `token` is the credential from the first `auth` frame, or the bearer token
/// from the upgrade request when one is present. Returning an error closes the
/// connection with [`CLOSE_UNAUTHORIZED`].
pub trait UpgradeAuthenticator: Send + Sync + 'static {
    fn authenticate(&self, token: Option<&str>) -> Result<Principal, AuthorizationError>;
}

/// Interim authenticator: mirror of the HTTP development principal.
///
/// It accepts any (or no) credential and yields the configured development
/// principal, so it grants no access when that principal is unset. Real token
/// validation (the NextAuth/BFF connection ticket) replaces this with the M2
/// identity work without changing the gateway.
#[derive(Clone, Debug, Default)]
pub struct DevAuthenticator {
    principal: Option<Principal>,
}

impl DevAuthenticator {
    #[must_use]
    pub fn new(principal: Option<Principal>) -> Self {
        Self { principal }
    }
}

impl UpgradeAuthenticator for DevAuthenticator {
    fn authenticate(&self, _token: Option<&str>) -> Result<Principal, AuthorizationError> {
        self.principal
            .clone()
            .ok_or(AuthorizationError::Unauthenticated)
    }
}
