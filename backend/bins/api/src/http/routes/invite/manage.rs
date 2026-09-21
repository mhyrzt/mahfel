use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::invite::{RevokeInviteLink, RotateInviteLink, revoke_link, rotate_link};
use mahfel_domain::InviteLinkId;
use time::OffsetDateTime;
use uuid::Uuid;

use super::dto::{InviteLinkResponse, RotateInviteLinkRequest};

/// Revoke an invite link. Revocation is terminal.
///
/// Manager-only, authorized against the link's own target, so a manager can only
/// revoke links belonging to a room they manage. A repeat revoke is a `409`.
#[utoipa::path(
    post,
    path = "/v1/invite-links/{link_id}/revoke",
    tag = "invite",
    params(("link_id" = String, Path, description = "Invite link identifier (UUID)")),
    responses(
        (status = 200, description = "Link revoked", body = InviteLinkResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Link does not exist or its target is not managed by the caller"),
        (status = 409, description = "Link is already revoked"),
    ),
)]
pub(crate) async fn revoke(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(link_id): ValidatedPath<Uuid>,
) -> Result<Json<InviteLinkResponse>, ApiError> {
    let now = OffsetDateTime::now_utc();
    let link = revoke_link(
        state.invites.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        RevokeInviteLink {
            actor: principal.actor_id,
            link_id: InviteLinkId::from_uuid(link_id),
        },
        now,
    )
    .await?;
    Ok(Json(InviteLinkResponse::from_link(&link, now)))
}

/// Rotate an invite link: retire the current code and issue a replacement.
///
/// Both happen in one transaction, so no client ever observes two live codes or
/// none. The previous row is retained so the history shows what was retired.
#[utoipa::path(
    post,
    path = "/v1/invite-links/{link_id}/rotate",
    tag = "invite",
    params(("link_id" = String, Path, description = "Invite link identifier (UUID)")),
    request_body = RotateInviteLinkRequest,
    responses(
        (status = 200, description = "Replacement link issued", body = InviteLinkResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Link does not exist or its target is not managed by the caller"),
        (status = 422, description = "Requested expiry is out of range"),
    ),
)]
pub(crate) async fn rotate(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(link_id): ValidatedPath<Uuid>,
    Json(request): Json<RotateInviteLinkRequest>,
) -> Result<Json<InviteLinkResponse>, ApiError> {
    let now = OffsetDateTime::now_utc();
    let link = rotate_link(
        state.invites.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        RotateInviteLink {
            actor: principal.actor_id,
            link_id: InviteLinkId::from_uuid(link_id),
            expiry_days: request.expiry_days,
        },
        now,
    )
    .await?;
    Ok(Json(InviteLinkResponse::from_link(&link, now)))
}
