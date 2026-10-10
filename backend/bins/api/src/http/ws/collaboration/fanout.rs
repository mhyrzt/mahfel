use mahfel_application::ports::CoordinationStore;
use mahfel_redis::RedisCoordination;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{broadcast, watch};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct AwarenessNotice {
    pub document_id: Uuid,
    pub origin: Uuid,
    pub payload: Vec<u8>,
}

pub(crate) struct Fanout {
    redis: RedisCoordination,
    node_id: Uuid,
    channels: Mutex<HashMap<Uuid, broadcast::Sender<AwarenessNotice>>>,
}

impl Fanout {
    pub(crate) fn spawn(
        redis: RedisCoordination,
        mut shutdown: watch::Receiver<bool>,
    ) -> Arc<Self> {
        let fanout = Arc::new(Self {
            redis,
            node_id: Uuid::new_v4(),
            channels: Mutex::new(HashMap::new()),
        });
        let sweeper = Arc::clone(&fanout);
        let mut sweep_shutdown = shutdown.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            loop {
                tokio::select! {
                    _ = sweep_shutdown.changed() => return,
                    _ = tick.tick() => sweeper.prune(),
                }
            }
        });
        let receiver = Arc::clone(&fanout);
        tokio::spawn(async move {
            let channel = receiver.redis.keys().channel("collaboration-awareness");
            loop {
                if *shutdown.borrow() {
                    return;
                }
                match receiver.redis.subscribe(&channel).await {
                    Ok(mut messages) => loop {
                        tokio::select! {
                            changed = shutdown.changed() => {
                                if changed.is_err() || *shutdown.borrow() { return; }
                            }
                            message = messages.recv() => {
                                let Some(message) = message else { break; };
                                if let Ok(notice) = serde_json::from_slice::<AwarenessNotice>(&message.payload)
                                    && notice.origin != receiver.node_id
                                {
                                    receiver.deliver(notice);
                                }
                            }
                        }
                    },
                    Err(error) => {
                        tracing::warn!(%error, "collaboration awareness subscription failed")
                    }
                }
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { return; }
                    }
                    () = tokio::time::sleep(Duration::from_secs(2)) => {}
                }
            }
        });
        fanout
    }

    pub(super) fn subscribe(&self, document_id: Uuid) -> broadcast::Receiver<AwarenessNotice> {
        let mut channels = self
            .channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        channels.retain(|_, sender| sender.receiver_count() > 0);
        channels
            .entry(document_id)
            .or_insert_with(|| broadcast::channel(64).0)
            .subscribe()
    }

    fn prune(&self) {
        self.channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|_, sender| sender.receiver_count() > 0);
    }

    fn deliver(&self, notice: AwarenessNotice) {
        let mut channels = self
            .channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        channels.retain(|_, sender| sender.receiver_count() > 0);
        if let Some(sender) = channels.get(&notice.document_id) {
            let _ = sender.send(notice);
        }
    }

    pub(super) async fn publish(&self, document_id: Uuid, payload: Vec<u8>) {
        let notice = AwarenessNotice {
            document_id,
            origin: self.node_id,
            payload,
        };
        self.deliver(notice.clone());
        let channel = self.redis.keys().channel("collaboration-awareness");
        if let Ok(bytes) = serde_json::to_vec(&notice)
            && let Err(error) = self.redis.publish(&channel, &bytes).await
        {
            tracing::warn!(%error, "collaboration awareness publish failed");
        }
    }
}
