//! Durable Mahfel business concepts: identifiers, value objects, aggregate
//! state machines, and their invariants.
//!
//! This crate is independent of infrastructure. Items are re-exported at the
//! crate root (`mahfel_domain::User`, `mahfel_domain::Room`, …) and organized
//! into modules by aggregate.

mod actor;
mod admission;
mod identity;
mod ids;
mod invite;
mod participant;
mod participation;
mod room;
mod session;
mod status;
mod user;
mod validation;

pub use actor::{
    ActorId, ActorKind, ServicePrincipal, ServicePrincipalError, ServicePrincipalStatus,
};
pub use admission::{
    AdmissionError, AdmissionOrigin, AdmissionPolicy, AdmissionRequest, AdmissionRequestId,
    AdmissionRequestStatus, MAX_PENDING_PER_SESSION,
};
pub use identity::{Identity, IdentityId};
pub use ids::NodeId;
pub use invite::{
    DEFAULT_EXPIRY_DAYS, InviteCode, InviteLink, InviteLinkError, InviteLinkId, InviteLinkStatus,
    InviteTarget, InviteTargetKind, MAX_EXPIRY_DAYS,
};
pub use participant::{Participant, ParticipantError, ParticipantId, ParticipantStatus};
pub use participation::{
    CorrectionReason, IntervalError, IntervalId, ParticipationInterval, intervals_overlap,
    total_connected_seconds,
};
pub use room::{
    MembershipError, RoleCapability, Room, RoomError, RoomId, RoomMember, RoomRole, RoomStatus,
    RoomVisibility, ensure_member_removable,
};
pub use session::{Session, SessionError, SessionId, SessionStatus, SessionVisibility};
pub use status::InvalidStatus;
pub use user::{User, UserError, UserStatus};
pub use validation::{
    AvatarUrl, DisplayName, EmailAddress, Issuer, MAX_AVATAR_URL_LEN, MAX_DISPLAY_NAME_LEN,
    MAX_EMAIL_LEN, MAX_ISSUER_LEN, MAX_ROOM_TITLE_LEN, MAX_SUBJECT_LEN, RoomTitle, Subject,
    ValidationError,
};
