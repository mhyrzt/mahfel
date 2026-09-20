//! Layered configuration for the Mahfel backend.
//!
//! Settings load from `default.toml`, an optional profile TOML, an optional
//! `local.toml`, and `MAHFEL_*` environment overrides (nested keys use `__`).
//! Each section is defined in its own module and re-exported here.

mod api;
mod database;
mod error;
mod keycloak;
mod media;
mod oidc;
mod redis;
mod settings;
mod telemetry;
mod turn;
mod worker;

pub use api::{ApiSettings, WsSettings};
pub use database::DatabaseSettings;
pub use error::SettingsError;
pub use keycloak::KeycloakSettings;
pub use media::MediaSettings;
pub use oidc::OidcSettings;
pub use redis::RedisSettings;
pub use settings::Settings;
pub use telemetry::TelemetrySettings;
pub use turn::TurnSettings;
pub use worker::WorkerSettings;
