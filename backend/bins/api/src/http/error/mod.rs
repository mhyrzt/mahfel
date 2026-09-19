//! Canonical HTTP error surface.
//!
//! [`ErrorCode`] is the status/machine-code catalog, [`ApiError`] is the JSON
//! envelope type, and [`api_error_from!`] wires application errors into it.

mod api_error;
mod code;
mod mapping;

pub use api_error::ApiError;
pub(crate) use mapping::api_error_from;
