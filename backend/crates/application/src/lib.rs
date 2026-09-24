//! Application use cases and outbound port contracts.
//!
//! Transports (`mahfel-api`, `mahfel-media`, `mahfel-worker`) depend on
//! this crate. Infrastructure adapters (`mahfel-postgres`,
//! `mahfel-media-client`, `mahfel-messaging`) implement the ports defined
//! here. This crate depends only on `mahfel-domain`.

pub mod admission;
pub mod chat;
pub mod error;
pub mod files;
pub mod invite;
pub mod jobs;
pub mod moderation;
pub mod participation;
pub mod placement;
pub mod policy;
pub mod ports;
pub mod rooms;
pub mod service_principals;
pub mod sessions;
pub mod users;

pub use error::{ApplicationError, ErrorKind};
