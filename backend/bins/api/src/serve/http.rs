use super::{SERVICE, VERSION, database::Database, telemetry::Telemetry};
use crate::http;
use crate::settings;
use anyhow::Result;
use axum::Router;
use mahfel_application::ports::AllowAuthenticated;
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_postgres::PostgresStores;
use std::sync::Arc;
use tokio::sync::watch;

/// Build the browser-facing router: state, authentication, and CORS.
pub(super) fn app(
    settings: &Settings,
    node_id: &NodeId,
    database: &Database,
    telemetry: &Telemetry,
    shutdown: watch::Receiver<bool>,
) -> Result<Router> {
    let origin = settings::allowed_origin(settings)?;
    let principal = settings::development_principal(settings)?;
    let stores = PostgresStores::new(database.pool.clone());
    let realtime = http::ws::WsState::new(
        Arc::new(http::ws::DevAuthenticator::new(principal.clone())),
        http::ws::WsConfig::from(&settings.api.ws),
        telemetry.ws_metrics.clone(),
        shutdown,
    );

    let state = http::AppState {
        service: SERVICE,
        version: VERSION,
        node_id: node_id.to_string(),
        auth: http::auth::AuthState::new(principal, Arc::new(AllowAuthenticated)),
        rooms: stores.rooms,
        realtime,
    };
    Ok(http::api_router(state).layer(http::cors_layer(origin)))
}
