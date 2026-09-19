use mahfel_config::Settings;
use mahfel_postgres::{ConnectionSettings, PoolSettings};
use std::time::Duration;

pub(crate) fn pool(settings: &Settings) -> PoolSettings {
    PoolSettings {
        min_connections: settings.database.min_connections,
        max_connections: settings.database.max_connections,
        acquire_timeout: Duration::from_secs(settings.database.acquire_timeout_seconds),
        idle_timeout: Duration::from_secs(settings.database.idle_timeout_seconds),
        max_lifetime: Duration::from_secs(settings.database.max_lifetime_seconds),
    }
}

pub(crate) fn connection(settings: &Settings) -> ConnectionSettings {
    ConnectionSettings {
        host: settings.database.host.clone(),
        port: settings.database.port,
        user: settings.database.user.clone(),
        password: settings.database.password.clone(),
        name: settings.database.name.clone(),
    }
}
