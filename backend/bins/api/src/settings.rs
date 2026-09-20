use anyhow::{Context, Result};
use axum::http::HeaderValue;
use mahfel_auth::TokenVerifierConfig;
use mahfel_config::Settings;
use mahfel_keycloak::KeycloakAdminSettings;
use mahfel_postgres::{ConnectionSettings, PoolSettings};
use mahfel_redis::RedisConnectionSettings;
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

/// Ephemeral coordination backend settings.
pub(crate) fn redis(settings: &Settings) -> RedisConnectionSettings {
    RedisConnectionSettings {
        url: settings.redis.url.clone(),
        key_prefix: settings.redis.key_prefix.clone(),
        connect_timeout: Duration::from_secs(settings.redis.connect_timeout_seconds),
        command_timeout: Duration::from_secs(settings.redis.command_timeout_seconds),
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

/// Verification configuration for the token Keycloak mints for `mahfel-api`.
pub(crate) fn jwt(settings: &Settings) -> TokenVerifierConfig {
    TokenVerifierConfig::new(
        settings.oidc.issuer_url.clone(),
        settings.oidc.client_id.clone(),
    )
}

/// Keycloak Admin API settings for linking local users to identities.
pub(crate) fn keycloak(settings: &Settings) -> KeycloakAdminSettings {
    KeycloakAdminSettings {
        base_url: settings.keycloak.base_url.clone(),
        realm: settings.keycloak.realm.clone(),
        client_id: settings.keycloak.admin_client_id.clone(),
        client_secret: settings.keycloak.admin_client_secret.clone(),
        http_timeout: Duration::from_secs(5),
    }
}
