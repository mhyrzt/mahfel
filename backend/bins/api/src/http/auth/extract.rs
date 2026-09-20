use super::{principal_from_claims, verification::AuthenticationError};
use crate::http::{error::ApiError, state::RoomStoreState};
use axum::{
    extract::FromRequestParts,
    http::{HeaderValue, header, request::Parts},
};
use mahfel_application::ports::Principal;
use mahfel_auth::AuthenticatedIdentity;

/// Maps an authentication failure to its HTTP status.
fn rejection(error: AuthenticationError) -> ApiError {
    match error {
        AuthenticationError::NotProvisioned => ApiError::forbidden("account is not provisioned"),
        _ => ApiError::unauthorized("authentication required"),
    }
}

/// Extractor that requires a verified, provisioned token.
///
/// Rejects with `401` when the credential is missing or invalid, and `403` when
/// the token has no linked application user.
pub struct Authenticated(pub Principal);

impl<R> FromRequestParts<crate::http::state::AppState<R>> for Authenticated
where
    R: RoomStoreState,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::http::state::AppState<R>,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts.headers.get(header::AUTHORIZATION));
        let claims = state
            .authentication
            .verify(token)
            .await
            .map_err(rejection)?;
        principal_from_claims(&claims).map(Self).map_err(rejection)
    }
}

/// Extractor that requires a verified token but accepts an unprovisioned one.
///
/// Used by `/v1/auth/provision`, which is the request that links the identity.
pub struct VerifiedToken(pub AuthenticatedIdentity);

impl<R> FromRequestParts<crate::http::state::AppState<R>> for VerifiedToken
where
    R: RoomStoreState,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::http::state::AppState<R>,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts.headers.get(header::AUTHORIZATION));
        state
            .authentication
            .verify(token)
            .await
            .map(Self)
            .map_err(rejection)
    }
}

/// Extract the bearer credential from an authorization header.
fn bearer_token(header: Option<&HeaderValue>) -> Option<&str> {
    header
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
}
