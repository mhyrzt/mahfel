use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::invite::{CreateInviteLink, create_invite_link};
use time::OffsetDateTime;
use uuid::Uuid;

use super::dto::{CreateRoomInviteLinkRequest, CreateSessionInviteLinkRequest, InviteLinkResponse};

/// Create an invite link for a room.
///
/// Manager-only: authorization resolves against the caller's own membership, so
/// a non-manager cannot issue a code by claiming a role it does not hold.
#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/invite-links",
    tag = "invite",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = CreateRoomInviteLinkRequest,
    responses(
        (status = 201, description = "Invite link created", body = InviteLinkResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the room"),
        (status = 404, description = "Room does not exist or is not visible to the caller"),
        (status = 422, description = "Requested expiry is out of range"),
    ),
)]
pub(crate) async fn create_for_room(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    Json(request): Json<CreateRoomInviteLinkRequest>,
) -> Result<(axum::http::StatusCode, Json<InviteLinkResponse>), ApiError> {
    let now = OffsetDateTime::now_utc();
    let link = create_invite_link(
        state.invites.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        CreateInviteLink {
            actor: principal.actor_id,
            target: super::dto::room_target(room_id),
            expiry_days: request.expiry_days,
        },
        now,
    )
    .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(InviteLinkResponse::from_link(&link, now)),
    ))
}

/// Create an invite link for a session.
#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/invite-links",
    tag = "invite",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    request_body = CreateSessionInviteLinkRequest,
    responses(
        (status = 201, description = "Invite link created", body = InviteLinkResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist or is not visible to the caller"),
        (status = 422, description = "Requested expiry is out of range"),
    ),
)]
pub(crate) async fn create_for_session(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
    Json(request): Json<CreateSessionInviteLinkRequest>,
) -> Result<(axum::http::StatusCode, Json<InviteLinkResponse>), ApiError> {
    let now = OffsetDateTime::now_utc();
    let link = create_invite_link(
        state.invites.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        CreateInviteLink {
            actor: principal.actor_id,
            target: super::dto::session_target(session_id),
            expiry_days: request.expiry_days,
        },
        now,
    )
    .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(InviteLinkResponse::from_link(&link, now)),
    ))
}
