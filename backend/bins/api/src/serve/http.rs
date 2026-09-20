use super::{SERVICE, VERSION, database::Database, telemetry::Telemetry};
use crate::http;
use crate::settings;
use anyhow::Result;
use axum::Router;
use mahfel_application::{
    ports::{RoleBasedAuthorizer, UserAttributeWriter},
    users::IdentityProvisioner,
};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_media_client::GrpcMediaGateway;
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
    let stores = PostgresStores::new(database.pool.clone());

    // One verifier serves both HTTP extraction and WebSocket upgrades.
    let authentication: std::sync::Arc<dyn http::auth::RequestAuthenticator> =
        Arc::new(http::auth::TokenAuthentication::new(
            mahfel_auth::TokenVerifier::new(settings::jwt(settings)),
        ));
    // Provisioning is the only path that touches identity storage; linking
    // writes the local user id onto the Keycloak identity.
    let provisioner: Arc<dyn IdentityProvisioner> = Arc::new(stores.identities.clone());
    let linker: Arc<dyn UserAttributeWriter> = Arc::new(mahfel_keycloak::KeycloakAdmin::new(
        settings::keycloak(settings),
    ));
    let media = Arc::new(
        GrpcMediaGateway::lazy(settings.api.media_grpc_url.clone())
            .map_err(|e| anyhow::anyhow!("invalid media grpc url: {e}"))?,
    );
    let realtime = http::ws::WsState::new(
        Arc::clone(&authentication),
        http::ws::WsConfig::from(&settings.api.ws),
        telemetry.ws_metrics.clone(),
        Arc::clone(&media),
        shutdown,
    );

    let state = http::AppState {
        service: SERVICE,
        version: VERSION,
        node_id: node_id.to_string(),
        auth: http::auth::AuthState::new(Arc::new(RoleBasedAuthorizer)),
        authentication,
        provisioner,
        linker,
        rooms: stores.rooms,
        realtime,
    };
    Ok(http::api_router(state).layer(http::cors_layer(origin)))
}
