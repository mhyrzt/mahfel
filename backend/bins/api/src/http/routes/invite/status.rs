use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_domain::AdmissionRequestId;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct InviteRequestStatusResponse {
    request_id: String,
    status: String,
}

#[utoipa::path(
    get,
    path = "/v1/invite-links/requests/{request_id}",
    tag = "invite",
    params(("request_id" = String, Path, description = "Admission request identifier (UUID)")),
    responses(
        (status = 200, description = "Caller's invite request status", body = InviteRequestStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Request does not exist or belongs to another principal"),
    ),
)]
pub(crate) async fn request_status(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(request_id): ValidatedPath<Uuid>,
) -> Result<Json<InviteRequestStatusResponse>, ApiError> {
    let request = state
        .admissions
        .get(AdmissionRequestId::from_uuid(request_id))
        .await
        .map_err(|_| ApiError::internal("admission request lookup failed"))?
        .filter(|request| request.actor_id == principal.actor_id)
        .ok_or_else(|| ApiError::not_found("admission request not found"))?;
    Ok(Json(InviteRequestStatusResponse {
        request_id: request.id.to_string(),
        status: request.status.as_str().to_owned(),
    }))
}
