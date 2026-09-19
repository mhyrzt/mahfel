use crate::{
    ApiSettings, DatabaseSettings, MediaSettings, OidcSettings, RedisSettings, SettingsError,
    TelemetrySettings, WorkerSettings,
};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: ApiSettings,
    pub oidc: OidcSettings,
    pub database: DatabaseSettings,
    pub media: MediaSettings,
    pub redis: RedisSettings,
    pub worker: WorkerSettings,
    pub telemetry: TelemetrySettings,
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        let config_dir = std::env::var_os("MAHFEL_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config"));
        let profile = std::env::var("MAHFEL_CONFIG_PROFILE").unwrap_or_else(|_| "docker".into());

        Config::builder()
            .add_source(File::from(config_dir.join("default.toml")))
            .add_source(File::from(config_dir.join(format!("{profile}.toml"))).required(false))
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
