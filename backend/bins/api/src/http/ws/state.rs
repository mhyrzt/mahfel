use super::{config::WsConfig, metrics::WsMetrics, registry::ConnectionRegistry};
use crate::http::auth::RequestAuthenticator;
use std::sync::Arc;
use tokio::sync::watch;

/// Shared WebSocket gateway state held by the router.
#[derive(Clone)]
pub struct WsState {
    pub registry: ConnectionRegistry,
    pub config: WsConfig,
    pub metrics: WsMetrics,
    pub authenticator: Arc<dyn RequestAuthenticator>,
    pub shutdown: watch::Receiver<bool>,
}

impl WsState {
    #[must_use]
    pub fn new(
        authenticator: Arc<dyn RequestAuthenticator>,
        config: WsConfig,
        metrics: WsMetrics,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        let registry = ConnectionRegistry::new(config.max_connections, metrics.clone());
        Self {
            registry,
            config,
            metrics,
            authenticator,
            shutdown,
        }
    }
}
