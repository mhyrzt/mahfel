use super::{
    SERVICE, VERSION, coordination::Coordination, database::Database, telemetry::Telemetry,
};
use crate::http;
use crate::settings;
use anyhow::Result;
use axum::Router;
use mahfel_application::{
    policy::ServerCapabilityResolver,
    ports::{FileStorage, UserAttributeWriter},
    users::IdentityProvisioner,
};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_media_client::GrpcMediaGateway;
use mahfel_postgres::{PostgresSessionLifecycle, PostgresStores};
use mahfel_storage::{S3FileStorage, S3Settings};
use std::sync::Arc;
use tokio::sync::watch;

/// Build the browser-facing router: state, authentication, and CORS.
pub(super) async fn app(
    settings: &Settings,
    node_id: &NodeId,
    database: &Database,
    coordination: &Coordination,
    telemetry: &Telemetry,
    shutdown: watch::Receiver<bool>,
) -> Result<Router> {
    let origin = settings::allowed_origin(settings)?;
    let stores = PostgresStores::new(database.pool.clone());
    // Advisory locks stay checked out while join/leave calls use the regular
    // stores. A separate small pool avoids deadlock when the store pool is
    // configured with a single connection.
    let mut lifecycle_pool_settings = settings::pool(settings);
    lifecycle_pool_settings.min_connections = 0;
    lifecycle_pool_settings.max_connections = 4;
    let lifecycle_pool =
        mahfel_postgres::connect(&settings::connection(settings), lifecycle_pool_settings).await?;
    let source_address = Arc::new(
        http::source_address::SourceAddressResolver::new(settings.api.trusted_proxy.as_deref())
            .await
            .map_err(|reason| anyhow::anyhow!("{reason}"))?,
    );

    // One verifier serves both HTTP extraction and WebSocket upgrades.
    let authentication: std::sync::Arc<dyn http::auth::RequestAuthenticator> =
        Arc::new(http::auth::TokenAuthentication::new(
            mahfel_auth::TokenVerifier::new(settings::jwt(settings)?),
        ));
    // Provisioning is the only path that touches identity storage; linking
    // writes the local user id onto the Keycloak identity.
    let provisioner: Arc<dyn IdentityProvisioner> = Arc::new(stores.identities.clone());
    let linker: Arc<dyn UserAttributeWriter> = Arc::new(mahfel_keycloak::KeycloakAdmin::new(
        settings::keycloak(settings),
    ));
    // Service principals authenticate with Mahfel-issued API keys; the same
    // store backs the admin provisioning surface.
    let service_auth: Arc<dyn http::auth::ServiceKeyAuthenticator> = Arc::new(
        http::auth::StoreServiceKeyAuthenticator::new(stores.service_principals.clone()),
    );
    let service_principals: Arc<dyn http::service_principals::ServicePrincipalAdmin> =
        Arc::new(http::service_principals::StoreServicePrincipalAdmin::new(
            stores.service_principals.clone(),
        ));
    let media = Arc::new(
        GrpcMediaGateway::authenticated(
            settings.api.media_grpc_url.clone(),
            &settings.api.control_token,
        )
        .map_err(|e| anyhow::anyhow!("invalid media grpc url: {e}"))?,
    );
    let rooms: Arc<dyn mahfel_application::ports::RoomStore> = Arc::new(stores.rooms.clone());
    let users: Arc<dyn mahfel_application::ports::UserStore> = Arc::new(stores.users.clone());
    let chat: Arc<dyn mahfel_application::ports::ChatStore> = Arc::new(stores.chat.clone());
    let interactions: Arc<dyn mahfel_application::ports::InteractionStore> =
        Arc::new(stores.interactions.clone());
    let collaboration: Arc<dyn mahfel_application::ports::CollaborationStore> =
        Arc::new(stores.collaboration.clone());
    let files: Arc<dyn mahfel_application::ports::FileStore> = Arc::new(stores.files.clone());
    let recording_storage = S3FileStorage::new(S3Settings::from(&settings.storage));
    let storage: Arc<dyn FileStorage> = Arc::new(recording_storage.clone());
    storage
        .check_bucket()
        .await
        .map_err(|error| anyhow::anyhow!("S3 bucket check failed: {error}"))?;
    let sessions: Arc<dyn mahfel_application::ports::SessionStore> =
        Arc::new(stores.sessions.clone());
    let participants: Arc<dyn mahfel_application::ports::ParticipantStore> =
        Arc::new(stores.participants.clone());
    let participation: Arc<dyn mahfel_application::ports::ParticipationStore> =
        Arc::new(stores.participation.clone());
    let admissions: Arc<dyn mahfel_application::ports::AdmissionStore> =
        Arc::new(stores.admissions.clone());
    let invites: Arc<dyn mahfel_application::ports::InviteStore> = Arc::new(stores.invites.clone());
    let blocks: Arc<dyn mahfel_application::ports::BlockStore> = Arc::new(stores.blocks.clone());
    let code_index: Arc<dyn mahfel_application::ports::CoordinationStore> =
        Arc::new(coordination.store.clone());
    let placements: Arc<dyn mahfel_application::ports::MediaPlacementStore> =
        Arc::new(stores.media_placements.clone());
    let leases: Arc<dyn mahfel_application::ports::MediaPlacementLease> =
        Arc::new(coordination.store.clone());
    let nodes: Arc<dyn mahfel_application::ports::MediaNodeRegistry> =
        Arc::new(coordination.store.clone());
    let placement = Arc::new(
        http::ws::MediaPlacementRouter::new(placements, leases, nodes, Arc::clone(&sessions))
            .with_control_token(settings.api.control_token.clone()),
    );
    let realtime = http::ws::WsState::new(
        Arc::clone(&authentication),
        http::ws::WsConfig::from(&settings.api.ws),
        telemetry.ws_metrics.clone(),
        Arc::clone(&media),
        http::ws::ParticipationStores {
            admissions: Arc::clone(&admissions),
            participants: Arc::clone(&participants),
            participation: Arc::clone(&participation),
            sessions: Arc::clone(&sessions),
            rooms: Arc::clone(&rooms),
            blocks: Arc::clone(&blocks),
        },
        shutdown.clone(),
    )
    .with_placement(Arc::clone(&placement))
    .with_chat(
        Arc::clone(&chat),
        Arc::clone(&files),
        coordination.store.clone(),
    )
    .with_collaboration(Arc::clone(&collaboration), coordination.store.clone())
    .with_session_presence(
        coordination.store.clone(),
        PostgresSessionLifecycle::new(lifecycle_pool),
    )
    .with_auth_tickets(coordination.store.clone())
    .with_interactions(Arc::clone(&interactions), coordination.store.clone());
    super::recordings::spawn(
        stores.recordings.clone(),
        recording_storage,
        realtime.clone(),
        shutdown.clone(),
    );
    realtime.speakers.spawn_placed(
        placement,
        realtime.registry.clone(),
        Arc::clone(&participants),
        Arc::clone(&sessions),
        shutdown,
    );

    let state = http::AppState {
        service: SERVICE,
        version: VERSION,
        node_id: node_id.to_string(),
        auth: http::auth::AuthState::new(Arc::new(ServerCapabilityResolver)),
        authentication,
        service_auth,
        service_principals,
        provisioner,
        linker,
        rooms,
        users,
        chat,
        interactions: Some(interactions),
        recording_exports: Some(Arc::new(stores.recordings.clone())),
        recordings: Some(Arc::new(stores.recordings.clone())),
        recording_playback: Some(Arc::new(stores.recordings.clone())),
        collaboration: Some(collaboration),
        files,
        storage,
        sessions,
        participation,
        admissions,
        invites,
        code_index,
        source_address,
        realtime,
        turn: settings.turn.clone(),
    };
    Ok(http::api_router(state).layer(http::cors_layer(origin)))
}
