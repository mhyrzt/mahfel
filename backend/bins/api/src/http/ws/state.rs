use super::{config::WsConfig, metrics::WsMetrics, registry::ConnectionRegistry};
use crate::http::auth::RequestAuthenticator;
use mahfel_application::ports::{ParticipantStore, RoomStore, SessionStore};
use mahfel_media_client::GrpcMediaGateway;
use std::sync::Arc;
use tokio::sync::watch;

/// The store seams the participation frames need, resolved together at join.
#[derive(Clone)]
pub struct ParticipationStores {
    pub participants: Arc<dyn ParticipantStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub rooms: Arc<dyn RoomStore>,
}

/// Shared WebSocket gateway state held by the router.
#[derive(Clone)]
pub struct WsState {
    pub registry: ConnectionRegistry,
    pub config: WsConfig,
    pub metrics: WsMetrics,
    pub authenticator: Arc<dyn RequestAuthenticator>,
    pub media: Arc<GrpcMediaGateway>,
    pub stores: ParticipationStores,
    pub shutdown: watch::Receiver<bool>,
}

impl WsState {
    #[must_use]
    pub fn new(
        authenticator: Arc<dyn RequestAuthenticator>,
        config: WsConfig,
        metrics: WsMetrics,
        media: Arc<GrpcMediaGateway>,
        stores: ParticipationStores,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        let registry = ConnectionRegistry::new(config.max_connections, metrics.clone());
        Self {
            registry,
            config,
            metrics,
            authenticator,
            media,
            stores,
            shutdown,
        }
    }
}
