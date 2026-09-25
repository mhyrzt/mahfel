use super::dto::OwnMembershipResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::policy::ServerCapabilityResolver;
use mahfel_application::ports::{Capability, CapabilityContext, CapabilityResolver};
use mahfel_domain::RoomId;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/membership",
    summary = "Get the caller's role in a room",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Caller's membership", body = OwnMembershipResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
    ),
)]
pub(crate) async fn own_membership(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<OwnMembershipResponse>, ApiError> {
    let member = state
        .rooms
        .get_member(RoomId::from_uuid(room_id), principal.actor_id)
        .await
        .map_err(|_| ApiError::internal("membership lookup failed"))?
        .ok_or_else(|| ApiError::not_found("room not found"))?;
    Ok(Json(OwnMembershipResponse {
        role: member.role.into(),
        can_manage_session: ServerCapabilityResolver
            .resolve(&principal, CapabilityContext::in_room(Some(member.role)))
            .has(Capability::ManageSession),
    }))
}
