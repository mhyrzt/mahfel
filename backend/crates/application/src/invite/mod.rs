//! Invite link management and the redemption primitives (M4-13).
//!
//! Management — create, list, revoke, rotate, regenerate — is capability-gated
//! and deny-by-default (AC 3). Redemption's confirmation flow is M4-14; what
//! lives here is what redemption leans on: code normalization, the
//! non-enumerating failure surface, and the rate limiter that bounds attempts.

pub mod code;
pub mod create;
pub mod error;
pub mod guard;
pub mod list;
pub mod rate_limit;
pub mod revoke;
pub mod rotate;

pub use code::insert_with_unique_code;
pub use create::{CreateInviteLink, InviteTargetSelector, create_invite_link};
pub use error::InviteOperationError;
pub use list::{InviteLinkQuery, list_links};
pub use rate_limit::{
    ATTEMPT_WINDOW, AttemptDecision, AttemptLimiter, MAX_ATTEMPTS_PER_ADDRESS,
    MAX_ATTEMPTS_PER_PRINCIPAL, RateLimitScope,
};
pub use revoke::{RevokeInviteLink, revoke_link};
pub use rotate::{RotateInviteLink, rotate_link};
