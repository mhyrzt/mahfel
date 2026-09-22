use super::{envelope::ServerMessage, metrics::WsMetrics};
use mahfel_domain::{ActorId, ParticipantId, RoomId, SessionId};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::{Mutex as AsyncMutex, mpsc};

use uuid::Uuid;

mod fanout;
mod lifecycle;
mod participation;
#[cfg(test)]
mod tests;

/// One item queued for a single connection.
pub enum Outbound {
    Message(ServerMessage),
}

/// Ephemeral connection state for one transport.
///
/// This is the transport half of participation: durable identity lives in
/// `mahfel.session_participants`, while "how many transports are attached and
/// which is newest" is volatile and lives here.
#[derive(Clone, Debug)]
struct Connection {
    sender: mpsc::Sender<Outbound>,
    actor_id: ActorId,
    session_id: Option<SessionId>,
    room_id: Option<RoomId>,
    participant_id: Option<ParticipantId>,
    /// Guards against a stale intent landing after a reconnect: an operation
    /// tagged with an older generation is a no-op. The guard is replica-local
    /// because this registry is process-local; cross-replica ordering belongs
    /// to the connection hub in M5-03 (TASK-34).
    generation: u64,
}

/// In-process registry of authenticated connections.
///
/// Fan-out is process-local. Cross-replica delivery is deliberately out of
/// scope until the M5-03 connection hub (TASK-34), which owns the
/// pub-sub-versus-streams decision and sequence/dedupe. Participation helpers
/// below inherit that scope: they see only connections on this replica.
#[derive(Clone)]
pub struct ConnectionRegistry {
    inner: Arc<Inner>,
}

struct Inner {
    max_connections: usize,
    reserved: AtomicUsize,
    connections: Mutex<HashMap<Uuid, Connection>>,
    generations: Mutex<HashMap<(SessionId, ActorId), u64>>,
    lifecycle_locks: Mutex<HashMap<LifecycleKey, Weak<LifecycleLock>>>,
    metrics: WsMetrics,
}

type LifecycleKey = (SessionId, ActorId);
type LifecycleLock = AsyncMutex<()>;

impl ConnectionRegistry {
    #[must_use]
    pub fn new(max_connections: usize, metrics: WsMetrics) -> Self {
        Self {
            inner: Arc::new(Inner {
                max_connections,
                reserved: AtomicUsize::new(0),
                connections: Mutex::new(HashMap::new()),
                generations: Mutex::new(HashMap::new()),
                lifecycle_locks: Mutex::new(HashMap::new()),
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
    pub fn register(
        &self,
        id: Uuid,
        actor_id: ActorId,
        sender: mpsc::Sender<Outbound>,
    ) -> RegisteredConnection {
        self.lock().insert(
            id,
            Connection {
                sender,
                actor_id,
                session_id: None,
                room_id: None,
                participant_id: None,
                generation: 0,
            },
        );
        RegisteredConnection {
            registry: self.clone(),
            id,
        }
    }

    fn unregister(&self, id: Uuid) {
        self.lock().remove(&id);
    }

    fn actor_of(&self, id: Uuid) -> Option<ActorId> {
        self.lock().get(&id).map(|connection| connection.actor_id)
    }

    fn generations(&self) -> MutexGuard<'_, HashMap<(SessionId, ActorId), u64>> {
        self.inner
            .generations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<Uuid, Connection>> {
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
