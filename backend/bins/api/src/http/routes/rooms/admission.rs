//! Room admission policy.

use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::admission::{RoomAdmission, set_room_admission};
use mahfel_domain::{AdmissionPolicy, RoomId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use super::dto::AdmissionPolicyDto;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SetAdmissionRequest {
    pub(crate) policy: AdmissionPolicyDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AdmissionPolicyResponse {
    pub(crate) room_id: String,
    pub(crate) policy: AdmissionPolicyDto,
}

/// Change whether joining this room's sessions requires approval.
///
/// Manager-only. Existing memberships are unaffected; the policy applies to
/// joins from now on.
#[utoipa::path(
    put,
    path = "/v1/rooms/{room_id}/admission",
    tag = "admission",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = SetAdmissionRequest,
    responses(
        (status = 200, description = "Admission policy updated", body = AdmissionPolicyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the room"),
        (status = 404, description = "Room does not exist or is not visible to the caller"),
        (status = 409, description = "Room is archived"),
    ),
)]
pub(crate) async fn set_admission(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    Json(request): Json<SetAdmissionRequest>,
) -> Result<Json<AdmissionPolicyResponse>, ApiError> {
    let policy: AdmissionPolicy = request.policy.into();
    set_room_admission(
        state.rooms.as_ref(),
        RoomAdmission {
            actor: principal.actor_id,
            room_id: RoomId::from_uuid(room_id),
            admission: policy,
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(AdmissionPolicyResponse {
        room_id: room_id.to_string(),
        policy: request.policy,
    }))
}

/// Read a room's admission policy.
#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/admission",
    tag = "admission",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Admission policy", body = AdmissionPolicyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the room"),
        (status = 404, description = "Room does not exist or is not visible to the caller"),
    ),
)]
pub(crate) async fn get_admission(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<AdmissionPolicyResponse>, ApiError> {
    let room = mahfel_application::rooms::get_room(
        state.rooms.as_ref(),
        RoomId::from_uuid(room_id),
        principal.actor_id,
    )
    .await?
    .ok_or_else(|| ApiError::not_found("room not found"))?;
    Ok(Json(AdmissionPolicyResponse {
        room_id: room_id.to_string(),
        policy: room.admission.into(),
    }))
}
