//! Durable Mahfel business concepts: identifiers, value objects, aggregate
//! state machines, and their invariants.
//!
//! This crate is independent of infrastructure. Items are re-exported at the
//! crate root (`mahfel_domain::User`, `mahfel_domain::Room`, …) and organized
//! into modules by aggregate.

mod ids;
mod room;
mod session;
mod status;
mod user;
mod validation;

pub use ids::NodeId;
pub use room::{
    MembershipError, Room, RoomError, RoomId, RoomMember, RoomRole, RoomStatus, RoomVisibility,
    ensure_member_removable,
};
pub use session::{ClassroomSession, ClassroomSessionId, SessionError, SessionStatus};
pub use status::InvalidStatus;
pub use user::{User, UserError, UserId, UserStatus};
pub use validation::{
    DisplayName, MAX_DISPLAY_NAME_LEN, MAX_ROOM_TITLE_LEN, RoomTitle, ValidationError,
};
