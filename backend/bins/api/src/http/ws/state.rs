use super::placement::MediaPlacementRouter;
use super::speaker::SpeakerProjection;
use super::{config::WsConfig, metrics::WsMetrics, registry::ConnectionRegistry};
use crate::http::auth::RequestAuthenticator;
use mahfel_application::ports::{
    AdmissionStore, BlockStore, ParticipantStore, ParticipationStore, RoomStore, SessionStore,
};
use mahfel_media_client::GrpcMediaGateway;
use std::sync::Arc;
use tokio::sync::watch;

/// The store seams the participation frames need, resolved together at join.
#[derive(Clone)]
pub struct ParticipationStores {
    pub admissions: Arc<dyn AdmissionStore>,
    pub participants: Arc<dyn ParticipantStore>,
    pub participation: Arc<dyn ParticipationStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub rooms: Arc<dyn RoomStore>,
    pub blocks: Arc<dyn BlockStore>,
}

/// Shared WebSocket gateway state held by the router.
#[derive(Clone)]
pub struct WsState {
    pub registry: ConnectionRegistry,
    pub speakers: SpeakerProjection,
    pub config: WsConfig,
    pub metrics: WsMetrics,
    pub authenticator: Arc<dyn RequestAuthenticator>,
    pub media: Arc<GrpcMediaGateway>,
    pub placement: Option<Arc<MediaPlacementRouter>>,
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
            speakers: SpeakerProjection::default(),
            config,
            metrics,
            authenticator,
            media,
            placement: None,
            stores,
            shutdown,
        }
    }

    pub fn with_placement(mut self, placement: Arc<MediaPlacementRouter>) -> Self {
        self.placement = Some(placement);
        self
    }

    pub async fn media_for(
        &self,
        session_id: mahfel_domain::SessionId,
        room_id: mahfel_domain::RoomId,
    ) -> Result<Arc<GrpcMediaGateway>, mahfel_application::placement::PlacementError> {
        match &self.placement {
            Some(router) => router.lookup(session_id, room_id).await,
            None => Ok(Arc::clone(&self.media)),
        }
    }

    pub fn control_media(&self) -> &dyn mahfel_application::ports::MediaGateway {
        match &self.placement {
            Some(router) => router.as_ref(),
            None => self.media.as_ref(),
        }
    }
}
