//! Moderation use cases (M4-05).
//!
//! Capability-gated actions that work for any vertical: a moderator mutes,
//! unmutes, removes, blocks, or unblocks. A vertical maps its vocabulary onto
//! the capabilities rather than adding engine roles (ADR 0020).
//!
//! Every action runs in one order, and the order is the property that matters:
//! resolve the capability, resolve the target against the authoritative store,
//! change the durable state, call the media plane, then audit. Capability first
//! so an unauthorized caller cannot probe for who exists; the state change
//! before enforcement so a media failure never reports a grant that did not
//! happen.

pub mod error;
pub mod mute;
pub mod reconcile;
pub mod remove;
mod scope;

#[cfg(test)]
mod tests;

pub use error::ModerationError;
pub use mute::{MuteCommand, MuteOutcome, MutedSource, mute, unmute};
pub use reconcile::{ReconcileOutcome, capability_for, reconcile_media};
pub use remove::{BlockCommand, BlockOutcome, RemoveCommand, block, remove_participant, unblock};
