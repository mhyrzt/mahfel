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
mod storage;
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
pub use storage::StorageSettings;
pub use telemetry::TelemetrySettings;
pub use turn::TurnSettings;
pub use worker::WorkerSettings;

mod recording;
pub use recording::RecordingSettings;

mod recording_export;
pub use recording_export::RecordingExportSettings;
