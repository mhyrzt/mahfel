//! Keycloak Admin API adapter.
//!
//! `mahfel-api` uses this to link a local user to its Keycloak identity by
//! writing the `mahfel_user_id` attribute, which the realm's protocol mapper
//! exposes as the `app_user_id` access-token claim (ADR 0019).

mod admin;
mod error;
mod settings;
mod writer;

pub use admin::KeycloakAdmin;
pub use error::KeycloakError;
pub use settings::KeycloakAdminSettings;
