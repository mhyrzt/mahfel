use super::envelope::ServerMessage;
use super::metrics::WsMetrics;
use mahfel_domain::{ActorId, ParticipantId, SessionId};
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
                generations: Mutex::new(HashMap::new()),
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

    /// Hand out the next generation for the connection's `(session, actor)`.
    pub fn joined(&self, id: Uuid, session_id: SessionId, participant_id: ParticipantId) -> u64 {
        let generation = {
            let actor_id = self.actor_of(id).unwrap_or_default();
            let mut generations = self.generations();
            let entry = generations.entry((session_id, actor_id)).or_insert(0);
            *entry = entry.wrapping_add(1);
            *entry
        };
        if let Some(connection) = self.lock().get_mut(&id) {
            connection.session_id = Some(session_id);
            connection.participant_id = Some(participant_id);
            connection.generation = generation;
        }
        generation
    }

    pub fn left(&self, id: Uuid) {
        if let Some(connection) = self.lock().get_mut(&id) {
            connection.session_id = None;
            connection.participant_id = None;
        }
    }

    #[must_use]
    pub fn participation(&self, id: Uuid) -> Option<(SessionId, ParticipantId)> {
        self.lock()
            .get(&id)
            .and_then(|connection| connection.session_id.zip(connection.participant_id))
    }

    /// How many connections this replica currently holds for a participant.
    ///
    /// Presence is derived from this count: a participant is present while at
    /// least one connection is live (AC 7). Replica-local.
    /// Enqueue a message for every connection in a session on this replica
    /// except the originating one.
    ///
    /// The originator already received a direct acknowledgement, and telling it
    /// that it joined when it *is* the participant is noise. Replica-local, so
    /// a session spanning replicas propagates only within each replica until
    /// the M5-03 fanout hub (TASK-34).
    pub fn broadcast_to_session_except(
        &self,
        session_id: SessionId,
        except: Uuid,
        message: &ServerMessage,
    ) {
        let targets: Vec<Uuid> = self
            .lock()
            .iter()
            .filter(|(id, connection)| **id != except && connection.session_id == Some(session_id))
            .map(|(id, _)| *id)
            .collect();
        if targets.is_empty() {
            return;
        }
        let connections = self.lock();
        for id in targets {
            if let Some(connection) = connections.get(&id) {
                match connection
                    .sender
                    .try_send(Outbound::Message(message.clone()))
                {
                    Ok(()) => {}
                    Err(mpsc::error::TrySendError::Full(_)) => self.inner.metrics.send_dropped(),
                    Err(mpsc::error::TrySendError::Closed(_)) => {}
                }
            }
        }
    }

    /// Enqueue a message for every authenticated connection.
    ///
    /// Uses `try_send` so a slow consumer cannot block the sender; overflow is
    /// counted and the stalled connection is closed by its own write timeout.
    pub fn broadcast(&self, message: &ServerMessage) {
        let connections = self.lock();
        for connection in connections.values() {
            match connection
                .sender
                .try_send(Outbound::Message(message.clone()))
            {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => self.inner.metrics.send_dropped(),
                Err(mpsc::error::TrySendError::Closed(_)) => {}
            }
        }
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
