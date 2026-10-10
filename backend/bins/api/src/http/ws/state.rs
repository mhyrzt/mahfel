use super::placement::MediaPlacementRouter;
use super::speaker::SpeakerProjection;
use super::{config::WsConfig, metrics::WsMetrics, registry::ConnectionRegistry};
use crate::http::auth::RequestAuthenticator;
use mahfel_application::ports::{
    AdmissionStore, BlockStore, ChatActivityStore, ChatStore, CollaborationStore, ParticipantStore,
    ParticipationStore, RoomStore, SessionStore,
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
    pub(super) moderation_origin: uuid::Uuid,
    pub speakers: SpeakerProjection,
    pub config: WsConfig,
    pub metrics: WsMetrics,
    pub authenticator: Arc<dyn RequestAuthenticator>,
    pub media: Arc<GrpcMediaGateway>,
    pub placement: Option<Arc<MediaPlacementRouter>>,
    pub stores: ParticipationStores,
    pub shutdown: watch::Receiver<bool>,
    pub chat: Option<Arc<dyn ChatStore>>,
    pub interactions: Option<Arc<dyn mahfel_application::ports::InteractionStore>>,
    interaction_wake: Arc<Notify>,
    interaction_redis: Option<RedisCoordination>,
    pub collaboration: Option<Arc<dyn CollaborationStore>>,
    pub collaboration_fanout: Option<Arc<super::collaboration::Fanout>>,
    pub chat_activity: Option<Arc<dyn ChatActivityStore>>,
    chat_redis: Option<RedisCoordination>,
    chat_wake: Arc<Notify>,
    pub session_presence: Option<RedisCoordination>,
    pub auth_tickets: Option<RedisCoordination>,
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
            moderation_origin: uuid::Uuid::new_v4(),
            speakers: SpeakerProjection::default(),
            config,
            metrics,
            authenticator,
            media,
            placement: None,
            stores,
            shutdown,
            chat: None,
            interactions: None,
            interaction_wake: Arc::new(Notify::new()),
            interaction_redis: None,
            collaboration: None,
            collaboration_fanout: None,
            chat_activity: None,
            chat_redis: None,
            chat_wake: Arc::new(Notify::new()),
            session_presence: None,
            auth_tickets: None,
            session_lifecycle: None,
            session_wake: Arc::new(Notify::new()),
        }
    }

    pub fn with_interactions(
        mut self,
        store: Arc<dyn mahfel_application::ports::InteractionStore>,
        redis: RedisCoordination,
    ) -> Self {
        self.interactions = Some(Arc::clone(&store));
        self.interaction_redis = Some(redis.clone());
        super::interaction_fanout::spawn(
            self.clone(),
            store,
            redis,
            Arc::clone(&self.interaction_wake),
        );
        self
    }
    pub async fn wake_interactions(&self) {
        self.interaction_wake.notify_one();
        if let Some(redis) = &self.interaction_redis {
            use mahfel_application::ports::CoordinationStore;
            if let Err(error) = redis
                .publish(&redis.keys().channel("session-interactions"), b"wake")
                .await
            {
                tracing::warn!(%error, "interaction wakeup lost; polling will recover");
            }
        }
    }

    pub fn with_chat(
        mut self,
        chat: Arc<dyn ChatStore>,
        files: Arc<dyn mahfel_application::ports::FileStore>,
        redis: RedisCoordination,
    ) -> Self {
        self.chat = Some(Arc::clone(&chat));
        let activity: Arc<dyn ChatActivityStore> = Arc::new(redis.clone());
        self.chat_activity = Some(Arc::clone(&activity));
        self.chat_redis = Some(redis.clone());
        super::chat_fanout::spawn(
            chat,
            activity,
            files,
            redis,
            self.registry.clone(),
            Arc::clone(&self.chat_wake),
            self.shutdown.clone(),
        );
        self
    }

    pub fn with_collaboration(
        mut self,
        collaboration: Arc<dyn CollaborationStore>,
        redis: RedisCoordination,
    ) -> Self {
        self.collaboration = Some(collaboration);
        self.collaboration_fanout = Some(super::collaboration::Fanout::spawn(
            redis,
            self.shutdown.clone(),
        ));
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
        super::moderation_fanout::spawn(self.clone());
        super::session_fanout::spawn(
            redis,
            self.registry.clone(),
            Arc::clone(&self.session_wake),
            self.shutdown.clone(),
        );
        self
    }

    pub fn with_auth_tickets(mut self, redis: RedisCoordination) -> Self {
        self.auth_tickets = Some(redis);
        self
    }

    pub async fn broadcast_moderation(
        &self,
        session: mahfel_domain::SessionId,
        message: super::ServerMessage,
    ) {
        super::moderation_fanout::publish(self, session, message).await;
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
