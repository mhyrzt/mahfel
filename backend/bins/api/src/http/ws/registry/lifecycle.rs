//! Serialize one actor's join and departure through durable interval changes.

use super::ConnectionRegistry;
use mahfel_domain::{ActorId, SessionId};
use std::sync::{Arc, PoisonError};
use tokio::sync::Mutex as AsyncMutex;

impl ConnectionRegistry {
    pub(crate) fn lifecycle_lock(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> Arc<AsyncMutex<()>> {
        let key = (session_id, actor_id);
        let mut locks = self
            .inner
            .lifecycle_locks
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = locks.get(&key).and_then(std::sync::Weak::upgrade) {
            return existing;
        }
        if locks.len() >= 1024 {
            locks.retain(|_, lock| lock.strong_count() > 0);
        }
        let lock = Arc::new(AsyncMutex::new(()));
        locks.insert(key, Arc::downgrade(&lock));
        lock
    }
}
