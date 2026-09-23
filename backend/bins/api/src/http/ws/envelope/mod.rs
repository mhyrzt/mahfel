//! Versioned JSON envelope for the realtime gateway.
//!
//! Every frame is `{ "v": <version>, "type": <kind>, ... }`. The version is
//! independent of the URL so the protocol can evolve without a new route.

mod client;
mod codec;
mod frame;
mod server;

pub use client::ClientMessage;
pub use codec::{ProtocolError, encode, encode_with_id, parse, parse_envelope};
pub use frame::{Envelope, PROTOCOL_VERSION};
pub use server::{ChatEventPayload, ServerMessage};
