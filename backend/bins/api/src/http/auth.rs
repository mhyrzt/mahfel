use crate::http::{
    error::{ApiError, api_error_from},
    state::RoomStoreState,
};
use axum::{extract::FromRequestParts, http::request::Parts};
use mahfel_application::ports::{Action, AuthorizationError, Authorizer, Principal};
use std::sync::Arc;

/// Authentication and policy for the API.
///
/// The principal is `None` when no authenticated identity is configured
/// (`development_principal_user_id` unset), which makes protected routes deny
/// by default.
#[derive(Clone)]
pub struct AuthState {
    principal: Option<Principal>,
    authorizer: Arc<dyn Authorizer>,
}

impl AuthState {
    #[must_use]
    pub fn new(principal: Option<Principal>, authorizer: Arc<dyn Authorizer>) -> Self {
        Self {
            principal,
            authorizer,
        }
    }

    /// Apply policy for an authenticated principal.
    pub fn authorize(
        &self,
        principal: &Principal,
        action: Action,
    ) -> Result<(), AuthorizationError> {
        self.authorizer.authorize(principal, action)
    }
}

/// Extractor that requires an authenticated principal.
///
/// Rejects with `401` and the JSON error envelope when no identity is
/// available, so a route can never silently run without authentication.
pub struct Authenticated(pub Principal);

impl<R> FromRequestParts<crate::http::state::AppState<R>> for Authenticated
where
    R: RoomStoreState,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &crate::http::state::AppState<R>,
    ) -> Result<Self, Self::Rejection> {
        state
            .auth
            .principal
            .clone()
            .map(Self)
            .ok_or_else(|| ApiError::unauthorized("authentication required"))
    }
}

api_error_from!(AuthorizationError);

/// Run a policy check, mapping denials into the HTTP error envelope.
pub fn authorize<R>(
    state: &crate::http::state::AppState<R>,
    principal: &Principal,
    action: Action,
) -> Result<(), ApiError> {
    state.auth.authorize(principal, action)?;
    Ok(())
}
