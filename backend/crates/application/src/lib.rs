//! Application use cases and outbound port contracts.
//!
//! Transports (`mahfel-api`, `mahfel-media`, `mahfel-worker`) depend on
//! this crate. Infrastructure adapters (`mahfel-postgres`,
//! `mahfel-media-client`, `mahfel-messaging`) implement the ports defined
//! here. This crate depends only on `mahfel-domain`.

pub mod error;
pub mod ports;
pub mod rooms;
pub mod sessions;
pub mod users;

pub use error::{ApplicationError, ErrorKind};
