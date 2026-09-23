use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use mahfel_application::{
    placement::{PlacementError, lookup_session, place_session},
    ports::{
        BoxFuture, CoordinationError, MediaGateway, MediaGatewayError, MediaNodeRecord,
        MediaNodeRegistry, MediaPlacementLease, MediaPlacementStore, SessionStore, StoreError,
    },
};
use mahfel_domain::{ActorId, NodeId, ProducerSource, RoomId, SessionId, SessionStatus};
use mahfel_media_client::GrpcMediaGateway;

const CREATE_ROOM_TIMEOUT: Duration = Duration::from_secs(5);

/// The SFU room is scoped to one session, even when a business room hosts
/// another session later. Browser-facing room identifiers remain unchanged.
pub(crate) fn media_room_id(session_id: SessionId) -> RoomId {
    RoomId::from_uuid(session_id.as_uuid())
}

/// Resolve the session's sticky media node and reuse its gRPC channel.
pub struct MediaPlacementRouter {
    placements: Arc<dyn MediaPlacementStore>,
    leases: Arc<dyn MediaPlacementLease>,
    nodes: Arc<dyn MediaNodeRegistry>,
    sessions: Arc<dyn SessionStore>,
    gateways: Mutex<HashMap<String, Arc<GrpcMediaGateway>>>,
}

impl MediaPlacementRouter {
    pub fn new(
        placements: Arc<dyn MediaPlacementStore>,
        leases: Arc<dyn MediaPlacementLease>,
        nodes: Arc<dyn MediaNodeRegistry>,
        sessions: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            placements,
            leases,
            nodes,
            sessions,
            gateways: Mutex::new(HashMap::new()),
        }
    }

    pub async fn place(
        &self,
        session_id: SessionId,
        room_id: RoomId,
    ) -> Result<Arc<GrpcMediaGateway>, PlacementError> {
        let placed = place_session(
            self.placements.as_ref(),
            self.leases.as_ref(),
            self.nodes.as_ref(),
            session_id,
            room_id,
        )
        .await?;
        let gateway = self.gateway(&placed.node)?;
        // Materialize the room before recording a participant. The next
        // heartbeat then counts it, even if the first transport is delayed.
        tokio::time::timeout(
            CREATE_ROOM_TIMEOUT,
            MediaGateway::create_room(gateway.as_ref(), media_room_id(session_id)),
        )
        .await
        .map_err(|_| PlacementError::Unavailable)?
        .map_err(|_| PlacementError::Unavailable)?;
        Ok(gateway)
    }

    pub async fn lookup(
        &self,
        session_id: SessionId,
        room_id: RoomId,
    ) -> Result<Arc<GrpcMediaGateway>, PlacementError> {
        let placed = lookup_session(
            self.placements.as_ref(),
            self.nodes.as_ref(),
            session_id,
            room_id,
        )
        .await?;
        self.gateway(&placed.node)
    }

    pub async fn live_nodes(&self) -> Result<Vec<MediaNodeRecord>, CoordinationError> {
        let nodes = self.nodes.live_nodes().await?;
        let urls: HashSet<_> = nodes.iter().map(|node| node.grpc_url.as_str()).collect();
        self.gateways
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|url, _| urls.contains(url.as_str()));
        Ok(nodes)
    }

    pub fn gateway_for_node(
        &self,
        node: &MediaNodeRecord,
    ) -> Result<Arc<GrpcMediaGateway>, PlacementError> {
        self.gateway(node)
    }

    pub async fn is_assigned(
        &self,
        session_id: SessionId,
        node_id: &NodeId,
    ) -> Result<bool, StoreError> {
        Ok(self
            .placements
            .get(session_id)
            .await?
            .is_some_and(|placement| placement.node_id == *node_id))
    }

    fn gateway(&self, node: &MediaNodeRecord) -> Result<Arc<GrpcMediaGateway>, PlacementError> {
        let url = node.grpc_url.clone();
        let mut gateways = self
            .gateways
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(gateway) = gateways.get(&url) {
            return Ok(Arc::clone(gateway));
        }
        let gateway =
            Arc::new(GrpcMediaGateway::lazy(url.clone()).map_err(|_| PlacementError::Unavailable)?);
        gateways.insert(url, Arc::clone(&gateway));
        Ok(gateway)
    }

    async fn room_gateway(
        &self,
        room_id: RoomId,
    ) -> Result<Option<(Arc<GrpcMediaGateway>, RoomId)>, MediaGatewayError> {
        let sessions = self
            .sessions
            .list_for_room(room_id)
            .await
            .map_err(MediaGatewayError::transport)?;
        let Some(session) = sessions
            .into_iter()
            .find(|session| session.status == SessionStatus::Live)
        else {
            return Ok(None);
        };
        self.lookup(session.id, room_id)
            .await
            .map(|gateway| Some((gateway, media_room_id(session.id))))
            .map_err(MediaGatewayError::transport)
    }
}

impl MediaGateway for MediaPlacementRouter {
    fn create_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            let (media, media_room) = self
                .room_gateway(room)
                .await?
                .ok_or_else(|| MediaGatewayError::transport("no live session"))?;
            MediaGateway::create_room(media.as_ref(), media_room).await
        })
    }

    fn close_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::close_room(media.as_ref(), media_room).await
            } else {
                Ok(())
            }
        })
    }

    fn pause_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        let producer_id = producer_id.to_owned();
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::pause_producer(media.as_ref(), media_room, &producer_id).await
            } else {
                Ok(())
            }
        })
    }

    fn resume_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        let producer_id = producer_id.to_owned();
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::resume_producer(media.as_ref(), media_room, &producer_id).await
            } else {
                Ok(())
            }
        })
    }

    fn close_participant(
        &self,
        room: RoomId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::close_participant(media.as_ref(), media_room, actor).await
            } else {
                Ok(())
            }
        })
    }

    fn pause_participant_source(
        &self,
        room: RoomId,
        actor: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::pause_participant_source(media.as_ref(), media_room, actor, source)
                    .await
            } else {
                Ok(0)
            }
        })
    }

    fn resume_participant_source(
        &self,
        room: RoomId,
        actor: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::resume_participant_source(media.as_ref(), media_room, actor, source)
                    .await
            } else {
                Ok(0)
            }
        })
    }

    fn close_participant_source(
        &self,
        room: RoomId,
        actor: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if let Some((media, media_room)) = self.room_gateway(room).await? {
                MediaGateway::close_participant_source(media.as_ref(), media_room, actor, source)
                    .await
            } else {
                Ok(0)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfus_name_rooms_by_session_not_business_room() {
        let business_room = RoomId::from_uuid(uuid::Uuid::from_u128(1));
        let first = SessionId::from_uuid(uuid::Uuid::from_u128(2));
        let second = SessionId::from_uuid(uuid::Uuid::from_u128(3));
        assert_ne!(media_room_id(first), business_room);
        assert_ne!(media_room_id(first), media_room_id(second));
        assert_eq!(media_room_id(first), media_room_id(first));
    }
}
