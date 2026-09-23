use super::placement::MediaPlacementRouter;
use super::speaker::SpeakerProjection;
use super::{config::WsConfig, metrics::WsMetrics, registry::ConnectionRegistry};
use crate::http::auth::RequestAuthenticator;
use mahfel_application::ports::{
    AdmissionStore, BlockStore, ChatActivityStore, ChatStore, ParticipantStore, ParticipationStore,
    RoomStore, SessionStore,
};
use mahfel_media_client::GrpcMediaGateway;
use mahfel_postgres::{PostgresSessionLifecycle, SessionLifecycleGuard};
use mahfel_redis::RedisCoordination;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, watch};

pub const SESSION_PRESENCE_TTL: Duration = Duration::from_secs(45);

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
    pub chat: Option<Arc<dyn ChatStore>>,
    pub chat_activity: Option<Arc<dyn ChatActivityStore>>,
    chat_redis: Option<RedisCoordination>,
    chat_wake: Arc<Notify>,
    pub session_presence: Option<RedisCoordination>,
    session_lifecycle: Option<PostgresSessionLifecycle>,
    session_wake: Arc<Notify>,
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
            chat: None,
            chat_activity: None,
            chat_redis: None,
            chat_wake: Arc::new(Notify::new()),
            session_presence: None,
            session_lifecycle: None,
            session_wake: Arc::new(Notify::new()),
        }
    }

    pub fn with_chat(mut self, chat: Arc<dyn ChatStore>, redis: RedisCoordination) -> Self {
        self.chat = Some(Arc::clone(&chat));
        let activity: Arc<dyn ChatActivityStore> = Arc::new(redis.clone());
        self.chat_activity = Some(Arc::clone(&activity));
        self.chat_redis = Some(redis.clone());
        super::chat_fanout::spawn(
            chat,
            activity,
            redis,
            self.registry.clone(),
            Arc::clone(&self.chat_wake),
            self.shutdown.clone(),
        );
        self
    }

    pub fn wake_chat(&self) {
        self.chat_wake.notify_one();
    }

    pub fn with_session_presence(
        mut self,
        redis: RedisCoordination,
        lifecycle: PostgresSessionLifecycle,
    ) -> Self {
        self.session_presence = Some(redis.clone());
        self.session_lifecycle = Some(lifecycle);
        super::session_fanout::spawn(
            redis,
            self.registry.clone(),
            Arc::clone(&self.session_wake),
            self.shutdown.clone(),
        );
        self
    }

    pub fn wake_session(&self) {
        self.session_wake.notify_one();
    }

    pub async fn lock_session_actor(
        &self,
        session_id: mahfel_domain::SessionId,
        actor_id: mahfel_domain::ActorId,
    ) -> Result<Option<SessionLifecycleGuard>, sqlx::Error> {
        match &self.session_lifecycle {
            Some(lifecycle) => lifecycle.lock(session_id, actor_id).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn publish_chat_wakeup(&self) {
        self.wake_chat();
        if let Some(redis) = &self.chat_redis {
            use mahfel_application::ports::CoordinationStore;
            let channel = redis.keys().channel("chat-events");
            if let Err(error) = redis.publish(&channel, b"wake").await {
                tracing::warn!(%error, "chat wakeup publish failed; polling will recover");
            }
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
