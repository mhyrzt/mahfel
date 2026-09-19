use super::{error::ApiError, pagination::PaginationParams};
use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use serde_json::json;

/// `ValidatedJson<T>` — JSON extractor that never leaks as 500.
///
/// Rejection is mapped to 422 with a stable envelope. Domain validation should
/// produce the same status through `ApiError::unprocessable`.
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let json = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                ApiError::unprocessable(rejection.to_string())
                    .with_details(json!({ "reason": rejection.to_string() }))
            })?;
        Ok(Self(json.0))
    }
}

/// `ValidatedQuery<T>` — query extractor that maps parse errors to 400.
#[allow(dead_code)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|query| Self(query.0))
            .map_err(|rejection| {
                ApiError::bad_request(rejection.to_string())
                    .with_details(json!({ "reason": rejection.to_string() }))
            })
    }
}

/// `ValidatedPagination` — convenience extractor for `PaginationParams`.
pub struct ValidatedPagination(pub PaginationParams);

impl<S> FromRequestParts<S> for ValidatedPagination
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<PaginationParams>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| {
                ApiError::bad_request(rejection.to_string())
                    .with_details(json!({ "reason": rejection.to_string() }))
            })?;
        params.validate().map(ValidatedPagination).map_err(|error| {
            ApiError::bad_request(error.to_string())
                .with_details(json!({ "reason": error.to_string() }))
        })
    }
}
