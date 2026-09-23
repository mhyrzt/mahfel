use crate::http::ws::registry::ConnectionRegistry;
use mahfel_redis::RedisCoordination;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Notify, watch},
    time::MissedTickBehavior,
};

/// Pub/sub reduces latency, while polling snapshots repairs any notice lost
/// during a Redis subscriber restart. No roster truth lives in the pub/sub
/// channel or in a replica's process memory.
pub(super) fn spawn(
    redis: RedisCoordination,
    registry: ConnectionRegistry,
    wake: Arc<Notify>,
    shutdown: watch::Receiver<bool>,
) {
    let listener = redis.clone();
    let listener_wake = Arc::clone(&wake);
    let listener_shutdown = shutdown.clone();
    tokio::spawn(async move {
        listen(listener, listener_wake, listener_shutdown).await;
    });
    tokio::spawn(async move {
        poll(redis, registry, wake, shutdown).await;
    });
}

async fn listen(redis: RedisCoordination, wake: Arc<Notify>, mut shutdown: watch::Receiver<bool>) {
    let channel = redis.keys().channel("session-presence");
    loop {
        if *shutdown.borrow() {
            return;
        }
        match redis.subscribe(&channel).await {
            Ok(mut messages) => loop {
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { return; }
                    }
                    message = messages.recv() => {
                        if message.is_none() { break; }
                        wake.notify_one();
                    }
                }
            },
            Err(error) => tracing::warn!(%error, "session presence subscription failed"),
        }
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
            () = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}

async fn poll(
    redis: RedisCoordination,
    registry: ConnectionRegistry,
    wake: Arc<Notify>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
            _ = interval.tick() => {}
            () = wake.notified() => {}
        }
        for session_id in registry.joined_sessions() {
            match redis.session_presence_snapshot(session_id).await {
                Ok(snapshot) => registry.send_session_roster(session_id, &snapshot),
                Err(error) => tracing::warn!(%error, "session roster snapshot failed"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::ws::{envelope::ServerMessage, metrics::WsMetrics, registry::Outbound};
    use mahfel_domain::{ActorId, ParticipantId, RoomId, SessionId};
    use mahfel_redis::{RedisConnectionSettings, RedisMetrics};
    use prometheus::Registry;
    use testcontainers::{
        GenericImage,
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
    };
    use tokio::sync::mpsc;
    use uuid::Uuid;

    async fn roster_with_count(receiver: &mut mpsc::Receiver<Outbound>, count: usize) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(Outbound::Message(ServerMessage::SessionRoster {
                    participant_ids, ..
                })) = receiver.recv().await
                    && participant_ids.len() == count
                {
                    return;
                }
            }
        })
        .await
        .expect("roster snapshot within timeout");
    }

    #[tokio::test]
    async fn two_replicas_resync_even_when_one_misses_pubsub()
    -> Result<(), Box<dyn std::error::Error>> {
        if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
            return Ok(());
        }
        let container = GenericImage::new("redis", "7.4-alpine")
            .with_exposed_port(6379.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .start()
            .await?;
        let port = container.get_host_port_ipv4(6379).await?;
        let settings = RedisConnectionSettings {
            url: format!("redis://127.0.0.1:{port}"),
            key_prefix: format!("mahfel-session-test-{}", Uuid::new_v4().simple()),
            connect_timeout: Duration::from_secs(2),
            command_timeout: Duration::from_secs(2),
        };
        let first =
            RedisCoordination::connect(&settings, RedisMetrics::register(&Registry::new())?)
                .await?;
        let second =
            RedisCoordination::connect(&settings, RedisMetrics::register(&Registry::new())?)
                .await?;
        let registry_a = ConnectionRegistry::new(16, WsMetrics::register(&Registry::new())?);
        let registry_b = ConnectionRegistry::new(16, WsMetrics::register(&Registry::new())?);
        let session = SessionId::new();
        let room = RoomId::new();
        let alice = ParticipantId::new();
        let bob = ParticipantId::new();
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let (tx_a, mut rx_a) = mpsc::channel(8);
        let (tx_b, mut rx_b) = mpsc::channel(8);
        let _a = registry_a.register(id_a, ActorId::new(), tx_a);
        let _b = registry_b.register(id_b, ActorId::new(), tx_b);
        registry_a.joined(id_a, room, session, alice);
        registry_b.joined(id_b, room, session, bob);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        spawn(
            first.clone(),
            registry_a,
            Arc::new(Notify::new()),
            shutdown_rx.clone(),
        );
        // Replica B has no subscriber. Its periodic snapshot must recover the
        // roster despite missing every pub/sub message.
        tokio::spawn(poll(
            second.clone(),
            registry_b,
            Arc::new(Notify::new()),
            shutdown_rx,
        ));
        let ttl = Duration::from_secs(10);
        first
            .session_presence_join(session, id_a, alice, ttl)
            .await?;
        second
            .session_presence_join(session, id_b, bob, ttl)
            .await?;
        roster_with_count(&mut rx_a, 2).await;
        roster_with_count(&mut rx_b, 2).await;
        assert_eq!(
            first
                .session_presence_leave(session, id_a, alice, ttl)
                .await?,
            0
        );
        roster_with_count(&mut rx_a, 1).await;
        roster_with_count(&mut rx_b, 1).await;
        shutdown_tx.send(true)?;
        Ok(())
    }
}
