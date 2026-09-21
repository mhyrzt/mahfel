use crate::http::error::{ApiError, api_error_from};
use mahfel_application::ports::{Action, AuthorizationError, Authorizer, Principal};
use std::sync::Arc;
/// Authorization policy for the API.
///
/// Authentication happens per request from the bearer token; this type only
/// holds the [`Authorizer`].
#[derive(Clone)]
pub struct AuthState {
    authorizer: Arc<dyn Authorizer>,
}

impl AuthState {
    #[must_use]
    pub fn new(authorizer: Arc<dyn Authorizer>) -> Self {
        Self { authorizer }
    }

    pub fn authorize(
        &self,
        principal: &Principal,
        action: Action,
    ) -> Result<(), AuthorizationError> {
        self.authorizer.authorize(principal, action)
    }
}

api_error_from!(AuthorizationError);

/// Run a policy check, mapping denials into the HTTP error envelope.
pub fn authorize(
    state: &crate::http::state::AppState,
    principal: &Principal,
    action: Action,
) -> Result<(), ApiError> {
    state.auth.authorize(principal, action)?;
    Ok(())
}
