use super::*;
use crate::http::ws::{envelope::ServerMessage, registry::Outbound};
use mahfel_application::ports::{
    BoxFuture, InteractionCommand, InteractionError, InteractionSnapshot, Principal, RaisedHand,
};
use mahfel_domain::{ActorId, ParticipantId, RoomId};
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::mpsc;
use uuid::Uuid;

struct VersionStore(AtomicI64);
impl InteractionStore for VersionStore {
    fn snapshot<'a>(
        &'a self,
        _: SessionId,
        _: &'a Principal,
    ) -> BoxFuture<'a, Result<InteractionSnapshot, InteractionError>> {
        Box::pin(async { Err(InteractionError::Forbidden) })
    }
    fn execute<'a>(
        &'a self,
        _: SessionId,
        _: &'a Principal,
        _: Uuid,
        _: InteractionCommand,
    ) -> BoxFuture<'a, Result<(), InteractionError>> {
        Box::pin(async { Err(InteractionError::Forbidden) })
    }
    fn version(&self, _: SessionId) -> BoxFuture<'_, Result<i64, InteractionError>> {
        Box::pin(async { Ok(self.0.load(Ordering::SeqCst)) })
    }
    fn clear_departed(
        &self,
        _: SessionId,
        _: ParticipantId,
    ) -> BoxFuture<'_, Result<(), InteractionError>> {
        Box::pin(async { Err(InteractionError::Forbidden) })
    }
    fn raised_hands(
        &self,
        _: SessionId,
    ) -> BoxFuture<'_, Result<Vec<RaisedHand>, InteractionError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

async fn receive_version(receiver: &mut mpsc::Receiver<Outbound>, expected: i64) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(Outbound::Message(ServerMessage::SessionInteractionsChanged {
                version,
                ..
            })) = receiver.recv().await
                && version == expected
            {
                return;
            }
        }
    })
    .await
    .expect("authoritative version delivered without pubsub");
}

#[tokio::test]
async fn replicas_recover_interaction_updates_without_any_pubsub_wakeup() {
    let store = Arc::new(VersionStore(AtomicI64::new(1)));
    let mut first = crate::http::tests::ws_state(None);
    let mut second = crate::http::tests::ws_state(None);
    let (shutdown, receiver) = watch::channel(false);
    first.shutdown = receiver.clone();
    second.shutdown = receiver;
    let session = SessionId::new();
    let room = RoomId::new();
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let (tx_a, mut rx_a) = mpsc::channel(4);
    let (tx_b, mut rx_b) = mpsc::channel(4);
    let _a = first.registry.register(id_a, ActorId::new(), tx_a);
    let _b = second.registry.register(id_b, ActorId::new(), tx_b);
    first
        .registry
        .joined(id_a, room, session, ParticipantId::new());
    second
        .registry
        .joined(id_b, room, session, ParticipantId::new());
    let task_a = tokio::spawn(poll(first, store.clone(), Arc::new(Notify::new())));
    let task_b = tokio::spawn(poll(second, store.clone(), Arc::new(Notify::new())));
    receive_version(&mut rx_a, 1).await;
    receive_version(&mut rx_b, 1).await;
    store.0.store(2, Ordering::SeqCst);
    receive_version(&mut rx_a, 2).await;
    receive_version(&mut rx_b, 2).await;
    shutdown.send(true).expect("shutdown pollers");
    task_a.await.expect("first poller stopped");
    task_b.await.expect("second poller stopped");
}
