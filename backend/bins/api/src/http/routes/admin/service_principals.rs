//! Administrative management of service principals.
//!
//! Every endpoint requires [`Capability::ManageSystem`]. A service principal holds
//! no global roles, so it can never manage other principals; only an
//! administrator token can. Raw API keys are returned once, on create and
//! rotate, and are never logged.

pub(crate) mod dto;

use crate::http::{
    auth::{self, Authenticated},
    error::{ApiError, api_error_from},
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
};
use dto::{
    CreateServicePrincipalRequest, CreatedServicePrincipalResponse,
    RotatedServicePrincipalKeyResponse, ServicePrincipalResponse,
};
use mahfel_application::ports::Capability;
use mahfel_application::service_principals::{
    CreateServicePrincipal, CreateServicePrincipalError, ListServicePrincipalsError,
    RevokeServicePrincipalError, RotateServicePrincipalKey, RotateServicePrincipalKeyError,
};
use uuid::Uuid;

api_error_from!(
    CreateServicePrincipalError,
    ListServicePrincipalsError,
    RotateServicePrincipalKeyError,
    RevokeServicePrincipalError,
);

#[utoipa::path(
    post,
    path = "/v1/admin/service-principals",
    operation_id = "adminCreateServicePrincipal",
    tag = "admin",
    request_body = CreateServicePrincipalRequest,
    responses(
        (status = 201, description = "Service principal created; api_key is shown once", body = CreatedServicePrincipalResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
        (status = 409, description = "Name already exists"),
        (status = 422, description = "Invalid name"),
    ),
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedJson(body): ValidatedJson<CreateServicePrincipalRequest>,
) -> Result<(StatusCode, Json<CreatedServicePrincipalResponse>), ApiError> {
    auth::authorize_global(&state, &principal, Capability::ManageSystem)?;
    let created = state
        .service_principals
        .create(CreateServicePrincipal {
            name: body.name,
            display_name: body.display_name,
        })
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedServicePrincipalResponse::from(created)),
    ))
}

#[utoipa::path(
    get,
    path = "/v1/admin/service-principals",
    operation_id = "adminListServicePrincipals",
    tag = "admin",
    responses(
        (status = 200, description = "Service principals", body = [ServicePrincipalResponse]),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
    ),
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Vec<ServicePrincipalResponse>>, ApiError> {
    auth::authorize_global(&state, &principal, Capability::ManageSystem)?;
    let principals = state.service_principals.list().await?;
    Ok(Json(
        principals
            .iter()
            .map(ServicePrincipalResponse::from)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/v1/admin/service-principals/{id}/rotate",
    operation_id = "adminRotateServicePrincipalKey",
    tag = "admin",
    params(("id" = String, Path, description = "Service principal identifier (UUID)")),
    responses(
        (status = 200, description = "New api_key shown once; previous keys revoked", body = RotatedServicePrincipalKeyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
        (status = 404, description = "Service principal not found"),
        (status = 409, description = "Service principal is revoked"),
    ),
)]
pub(crate) async fn rotate(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(id): ValidatedPath<Uuid>,
) -> Result<Json<RotatedServicePrincipalKeyResponse>, ApiError> {
    auth::authorize_global(&state, &principal, Capability::ManageSystem)?;
    let rotated = state
        .service_principals
        .rotate(RotateServicePrincipalKey {
            id: mahfel_domain::ActorId::from_uuid(id),
        })
        .await?;
    Ok(Json(RotatedServicePrincipalKeyResponse::from(rotated)))
}

#[utoipa::path(
    delete,
    path = "/v1/admin/service-principals/{id}",
    operation_id = "adminRevokeServicePrincipal",
    tag = "admin",
    params(("id" = String, Path, description = "Service principal identifier (UUID)")),
    responses(
        (status = 204, description = "Service principal revoked"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
        (status = 404, description = "Service principal not found"),
    ),
)]
pub(crate) async fn revoke(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(id): ValidatedPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    auth::authorize_global(&state, &principal, Capability::ManageSystem)?;
    state
        .service_principals
        .revoke(mahfel_domain::ActorId::from_uuid(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/admin/service-principals", get(list).post(create))
        .route("/admin/service-principals/{id}/rotate", post(rotate))
        .route("/admin/service-principals/{id}", delete(revoke))
}
