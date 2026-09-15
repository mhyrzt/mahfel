use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: ApiSettings,
    pub database: DatabaseSettings,
    pub media: MediaSettings,
    pub worker: WorkerSettings,
    pub telemetry: TelemetrySettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiSettings {
    pub http_bind: String,
    pub operations_bind: String,
    pub media_grpc_url: String,
    pub allowed_origin: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub migrate_on_start: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MediaSettings {
    pub grpc_bind: String,
    pub operations_bind: String,
    pub worker_count: usize,
    pub rtc_port_min: u16,
    pub rtc_port_max: u16,
    pub listen_ip: String,
    pub announced_address: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerSettings {
    pub operations_bind: String,
    pub amqp_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetrySettings {
    pub log_filter: String,
    pub otlp_endpoint: String,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("configuration could not be loaded: {0}")]
    Load(#[from] config::ConfigError),
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        let config_dir = std::env::var_os("MAHFEL_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config"));

        Config::builder()
            .add_source(File::from(config_dir.join("default.toml")))
            .add_source(File::from(config_dir.join("local.toml")).required(false))
            .add_source(
                Environment::with_prefix("MAHFEL")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
            .map_err(SettingsError::from)
    }
}
