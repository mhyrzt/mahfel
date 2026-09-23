//! Capability-gated invite links and non-enumerating redemption.

pub mod code;
pub mod create;
pub mod error;
pub mod guard;
pub mod list;
pub mod rate_limit;
pub mod redeem;
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
pub use redeem::{RedeemError, RedeemInviteLink, Redeemed, redeem_invite_link};
pub use revoke::{RevokeInviteLink, revoke_link};
pub use rotate::{RotateInviteLink, rotate_link};

#[cfg(test)]
mod tests;
