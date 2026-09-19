use super::envelope::ServerMessage;
use super::metrics::WsMetrics;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::mpsc;
use uuid::Uuid;

/// One item queued for a single connection.
pub enum Outbound {
    Message(ServerMessage),
}

/// In-process registry of authenticated connections.
///
/// Fan-out is process-local. Cross-replica delivery is deliberately out of
/// scope until the M1-06 Redis coordination adapter.
#[derive(Clone)]
pub struct ConnectionRegistry {
    inner: Arc<Inner>,
}

struct Inner {
    max_connections: usize,
    reserved: AtomicUsize,
    connections: Mutex<HashMap<Uuid, mpsc::Sender<Outbound>>>,
    metrics: WsMetrics,
}

impl ConnectionRegistry {
    #[must_use]
    pub fn new(max_connections: usize, metrics: WsMetrics) -> Self {
        Self {
            inner: Arc::new(Inner {
                max_connections,
                reserved: AtomicUsize::new(0),
                connections: Mutex::new(HashMap::new()),
                metrics,
            }),
        }
    }

    /// Reserve a connection slot before the upgrade completes.
    ///
    /// Returns `None` once the process is at capacity, so the caller can shed
    /// load before performing the handshake.
    pub fn try_acquire(&self) -> Option<ConnectionGuard> {
        let mut current = self.inner.reserved.load(Ordering::Relaxed);
        loop {
            if current >= self.inner.max_connections {
                self.inner.metrics.connection_rejected();
                return None;
            }
            match self.inner.reserved.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.inner.metrics.connection_opened();
                    return Some(ConnectionGuard {
                        registry: self.clone(),
                    });
                }
                Err(actual) => current = actual,
            }
        }
    }

    fn release(&self) {
        self.inner.reserved.fetch_sub(1, Ordering::AcqRel);
        self.inner.metrics.connection_closed();
    }

    /// Add an authenticated connection to the fan-out set.
    pub fn register(&self, id: Uuid, sender: mpsc::Sender<Outbound>) -> RegisteredConnection {
        self.lock().insert(id, sender);
        RegisteredConnection {
            registry: self.clone(),
            id,
        }
    }

    fn unregister(&self, id: Uuid) {
        self.lock().remove(&id);
    }

    /// Enqueue a message for every authenticated connection.
    ///
    /// Uses `try_send` so a slow consumer cannot block the sender; overflow is
    /// counted and the stalled connection is closed by its own write timeout.
    pub fn broadcast(&self, message: &ServerMessage) {
        let connections = self.lock();
        for sender in connections.values() {
            match sender.try_send(Outbound::Message(message.clone())) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => self.inner.metrics.send_dropped(),
                Err(mpsc::error::TrySendError::Closed(_)) => {}
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<Uuid, mpsc::Sender<Outbound>>> {
        self.inner
            .connections
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Holds a capacity reservation; released when the connection ends.
pub struct ConnectionGuard {
    registry: ConnectionRegistry,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.registry.release();
    }
}

/// Removes a connection from the fan-out set when dropped.
pub struct RegisteredConnection {
    registry: ConnectionRegistry,
    id: Uuid,
}

impl RegisteredConnection {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl Drop for RegisteredConnection {
    fn drop(&mut self) {
        self.registry.unregister(self.id);
    }
}
