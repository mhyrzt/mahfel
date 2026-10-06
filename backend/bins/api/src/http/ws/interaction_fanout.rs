use super::WsState;
use mahfel_application::ports::InteractionStore;
use mahfel_domain::SessionId;
use mahfel_redis::RedisCoordination;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Notify, watch},
    time::MissedTickBehavior,
};

pub(super) fn spawn(
    state: WsState,
    store: Arc<dyn InteractionStore>,
    redis: RedisCoordination,
    wake: Arc<Notify>,
) {
    let listener = redis.clone();
    let listener_wake = Arc::clone(&wake);
    let shutdown = state.shutdown.clone();
    let registry = state.registry.clone();
    tokio::spawn(async move {
        listen(listener, listener_wake, registry, shutdown).await;
    });
    tokio::spawn(poll(state, store, wake));
}

async fn poll(state: WsState, store: Arc<dyn InteractionStore>, wake: Arc<Notify>) {
    let mut shutdown = state.shutdown.clone();
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            changed = shutdown.changed() => { if changed.is_err() || *shutdown.borrow() { return; } }
            _ = interval.tick() => {}
            () = wake.notified() => {}
        }
        for session in state.registry.joined_sessions() {
            if let Err(error) = reconcile_hands(&state, store.as_ref(), session).await {
                tracing::warn!(%error, "raised hand reconciliation failed");
            }
            match store.version(session).await {
                Ok(version) => state.registry.send_interaction_version(session, version),
                Err(error) => tracing::warn!(%error, "interaction version lookup failed"),
            }
        }
    }
}

async fn reconcile_hands(
    state: &WsState,
    store: &dyn InteractionStore,
    session: SessionId,
) -> Result<(), mahfel_application::ports::InteractionError> {
    let Some(redis) = &state.session_presence else {
        return Ok(());
    };
    let presence = redis
        .session_presence_snapshot(session)
        .await
        .map_err(mahfel_application::ports::StoreError::unexpected)?;
    for hand in store.raised_hands(session).await? {
        if !presence.participant_ids.contains(&hand.participant_id) {
            let _guard = state
                .lock_session_actor(session, hand.actor_id)
                .await
                .map_err(mahfel_application::ports::StoreError::unexpected)?;
            let current = redis
                .session_presence_snapshot(session)
                .await
                .map_err(mahfel_application::ports::StoreError::unexpected)?;
            if !current.participant_ids.contains(&hand.participant_id) {
                store.clear_departed(session, hand.participant_id).await?;
            }
        }
    }
    Ok(())
}

async fn listen(
    redis: RedisCoordination,
    wake: Arc<Notify>,
    registry: super::registry::ConnectionRegistry,
    mut shutdown: watch::Receiver<bool>,
) {
    let channel = redis.keys().channel("session-interactions");
    loop {
        if *shutdown.borrow() {
            return;
        }
        match redis.subscribe(&channel).await {
            Ok(mut messages) => loop {
                tokio::select! {
                    changed = shutdown.changed() => { if changed.is_err() || *shutdown.borrow() { return; } }
                    message = messages.recv() => {
                        let Some(payload) = message else { break; };
                        let payload = payload.payload;
                        if payload == b"wake" {
                            wake.notify_one();
                        } else if payload.len() <= 4096 {
                            use super::envelope::ServerMessage;
                            if let Ok(message @ ServerMessage::SessionReaction { .. }) = serde_json::from_slice::<ServerMessage>(&payload)
                                && let ServerMessage::SessionReaction { ref session_id, .. } = message
                                && let Ok(session) = session_id.parse::<uuid::Uuid>() {
                                registry.broadcast_to_session(SessionId::from_uuid(session), &message);
                            }
                        }
                    }
                }
            },
            Err(error) => tracing::warn!(%error, "interaction subscription failed"),
        }
        tokio::select! {
            changed = shutdown.changed() => { if changed.is_err() || *shutdown.borrow() { return; } }
            () = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}

#[cfg(test)]
mod tests;
