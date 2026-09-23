use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use mahfel_domain::{NodeId, RoomId, SessionId};

use crate::ports::{
    BoxFuture, CoordinationError, MediaNodeRecord, MediaNodeRegistry, MediaPlacement,
    MediaPlacementLease, MediaPlacementStore, StoreError,
};

use super::{PlacementError, lookup_session, place_session};

#[derive(Default)]
struct Placements(Mutex<Option<MediaPlacement>>);

impl MediaPlacementStore for Placements {
    fn get(
        &self,
        _session_id: SessionId,
    ) -> BoxFuture<'_, Result<Option<MediaPlacement>, StoreError>> {
        Box::pin(async move { Ok(self.0.lock().expect("placement lock").clone()) })
    }

    fn insert_or_get<'a>(
        &'a self,
        candidate: &'a MediaPlacement,
    ) -> BoxFuture<'a, Result<MediaPlacement, StoreError>> {
        Box::pin(async move {
            let mut row = self.0.lock().expect("placement lock");
            Ok(row.get_or_insert_with(|| candidate.clone()).clone())
        })
    }
}

struct Lease {
    node: NodeId,
    calls: AtomicUsize,
    fail_first: bool,
}

impl MediaPlacementLease for Lease {
    fn acquire<'a>(
        &'a self,
        _session_id: SessionId,
        _candidates: &'a [NodeId],
        _ttl: Duration,
    ) -> BoxFuture<'a, Result<Option<NodeId>, CoordinationError>> {
        Box::pin(async move {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            Ok((!self.fail_first || call > 0).then(|| self.node.clone()))
        })
    }
}

#[derive(Default)]
struct Nodes(Mutex<Vec<MediaNodeRecord>>);

impl MediaNodeRegistry for Nodes {
    fn heartbeat<'a>(
        &'a self,
        record: &'a MediaNodeRecord,
        _ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move {
            let mut nodes = self.0.lock().expect("node lock");
            nodes.retain(|node| node.node_id != record.node_id);
            nodes.push(record.clone());
            Ok(())
        })
    }

    fn live_nodes(&self) -> BoxFuture<'_, Result<Vec<MediaNodeRecord>, CoordinationError>> {
        Box::pin(async move { Ok(self.0.lock().expect("node lock").clone()) })
    }

    fn live_node<'a>(
        &'a self,
        node_id: &'a NodeId,
    ) -> BoxFuture<'a, Result<Option<MediaNodeRecord>, CoordinationError>> {
        Box::pin(async move {
            Ok(self
                .0
                .lock()
                .expect("node lock")
                .iter()
                .find(|node| node.node_id == *node_id)
                .cloned())
        })
    }
}

fn node(id: &str) -> MediaNodeRecord {
    MediaNodeRecord {
        node_id: NodeId::from_deployment(id).expect("valid node id"),
        grpc_url: format!("http://{id}:50051"),
        max_rooms: 2,
        active_workers: 1,
        active_rooms: 0,
        transports: 0,
        draining: false,
    }
}

#[tokio::test]
async fn a_lost_node_never_moves_a_live_session() {
    let a = node("media-a");
    let b = node("media-b");
    let store = Placements::default();
    let lease = Lease {
        node: a.node_id.clone(),
        calls: AtomicUsize::new(0),
        fail_first: false,
    };
    let registry = Nodes(Mutex::new(vec![a.clone(), b.clone()]));
    let session_id = SessionId::new();
    let room_id = RoomId::new();

    let first = place_session(&store, &lease, &registry, session_id, room_id)
        .await
        .expect("place");
    assert_eq!(first.node.node_id, a.node_id);
    *registry.0.lock().expect("node lock") = vec![b];
    assert!(matches!(
        lookup_session(&store, &registry, session_id, room_id).await,
        Err(PlacementError::Unavailable)
    ));
    assert!(matches!(
        place_session(&store, &lease, &registry, session_id, room_id).await,
        Err(PlacementError::Unavailable)
    ));
    assert_eq!(
        lease.calls.load(Ordering::SeqCst),
        1,
        "no second claim after node loss"
    );
    assert_eq!(
        store
            .get(session_id)
            .await
            .expect("read")
            .expect("row")
            .node_id,
        a.node_id
    );
    *registry.0.lock().expect("node lock") = vec![a.clone()];
    assert_eq!(
        lookup_session(&store, &registry, session_id, room_id)
            .await
            .expect("retry")
            .node
            .node_id,
        a.node_id
    );
}

#[tokio::test]
async fn an_unclaimed_failure_can_retry_without_a_durable_decision() {
    let a = node("media-a");
    let store = Arc::new(Placements::default());
    let lease = Lease {
        node: a.node_id.clone(),
        calls: AtomicUsize::new(0),
        fail_first: true,
    };
    let registry = Nodes(Mutex::new(vec![a]));
    let session_id = SessionId::new();
    let room_id = RoomId::new();

    assert!(matches!(
        place_session(store.as_ref(), &lease, &registry, session_id, room_id).await,
        Err(PlacementError::Unavailable)
    ));
    assert!(store.get(session_id).await.expect("read").is_none());
    assert!(
        place_session(store.as_ref(), &lease, &registry, session_id, room_id)
            .await
            .is_ok()
    );
    assert_eq!(lease.calls.load(Ordering::SeqCst), 2);
}
