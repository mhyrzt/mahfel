use mahfel_application::invite::InviteTargetSelector;
use mahfel_domain::{InviteLink, InviteTargetKind, RoomId, SessionId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// `utoipa` never reaches `mahfel-domain` (ADR 0015), so the domain's tagged
/// target enum becomes an explicit kind plus id here.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InviteTargetKindDto {
    Room,
    Session,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateRoomInviteLinkRequest {
    /// Defaults to 7, maximum 30.
    pub(crate) expiry_days: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateSessionInviteLinkRequest {
    /// Defaults to 7, maximum 30.
    pub(crate) expiry_days: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RotateInviteLinkRequest {
    /// Omit to preserve the previous link's window rather than resetting it.
    pub(crate) expiry_days: Option<i64>,
}

/// A link's code is not a secret to its manager, but it is never logged.
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct InviteLinkResponse {
    pub(crate) id: String,
    pub(crate) code: String,
    /// The `xxx-yyyy-zzz` form meant to be shown or copied.
    pub(crate) display_code: String,
    pub(crate) target_kind: InviteTargetKindDto,
    pub(crate) target_id: String,
    pub(crate) status: String,
    #[serde(with = "super::super::rfc3339")]
    pub(crate) expires_at: OffsetDateTime,
    #[serde(with = "super::super::rfc3339")]
    pub(crate) created_at: OffsetDateTime,
    #[serde(with = "super::super::rfc3339::option")]
    pub(crate) revoked_at: Option<OffsetDateTime>,
    pub(crate) created_by: String,
    /// Derived from status and expiry; the server remains authoritative.
    pub(crate) redeemable: bool,
}

impl InviteLinkResponse {
    pub(crate) fn from_link(link: &InviteLink, now: OffsetDateTime) -> Self {
        Self {
            id: link.id.to_string(),
            code: link.code.as_str().to_owned(),
            display_code: link.code.format(),
            target_kind: match link.target.kind() {
                InviteTargetKind::Room => InviteTargetKindDto::Room,
                InviteTargetKind::Session => InviteTargetKindDto::Session,
            },
            target_id: link.target.id().to_string(),
            status: link.status.as_str().to_owned(),
            expires_at: link.expires_at,
            created_at: link.created_at,
            revoked_at: link.revoked_at,
            created_by: link.created_by.to_string(),
            redeemable: link.is_redeemable_at(now),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct InviteLinkListResponse {
    pub(crate) links: Vec<InviteLinkResponse>,
}

/// Build the room-targeted selector used by both room routes.
pub(crate) fn room_target(room_id: uuid::Uuid) -> InviteTargetSelector {
    InviteTargetSelector::Room(RoomId::from_uuid(room_id))
}

/// Build the session-targeted selector used by both session routes.
pub(crate) fn session_target(session_id: uuid::Uuid) -> InviteTargetSelector {
    InviteTargetSelector::Session(SessionId::from_uuid(session_id))
}
