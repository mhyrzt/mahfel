use anyhow::{Context, Result};
use axum::http::HeaderValue;
use mahfel_application::ports::Principal;
use mahfel_config::Settings;
use mahfel_domain::UserId;
use mahfel_postgres::{ConnectionSettings, PoolSettings};
use std::time::Duration;
use uuid::Uuid;

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

/// Browser origin allowed by CORS.
pub(crate) fn allowed_origin(settings: &Settings) -> Result<HeaderValue> {
    settings
        .api
        .allowed_origin
        .parse::<HeaderValue>()
        .context("parse configured browser origin")
}

/// Local-only development principal, if configured.
pub(crate) fn development_principal(settings: &Settings) -> Result<Option<Principal>> {
    settings
        .api
        .development_principal_user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value.parse::<Uuid>().map(|id| Principal {
                user_id: UserId::from_uuid(id),
            })
        })
        .transpose()
        .context("parse development principal user id")
}
