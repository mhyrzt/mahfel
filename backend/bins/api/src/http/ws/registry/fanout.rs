//! Replica-local WebSocket fan-out and bounded outbound queues.

use super::{ConnectionRegistry, Outbound};
use crate::http::ws::envelope::ServerMessage;
use mahfel_domain::SessionId;
use mahfel_redis::SessionPresenceSnapshot;
use tokio::sync::mpsc;
use uuid::Uuid;

impl ConnectionRegistry {
    /// Every replica sends the same authoritative snapshot to its local
    /// sockets. Per-socket version tracking prevents repeated polling from
    /// filling outbound queues with duplicates.
    pub fn send_session_roster(&self, session_id: SessionId, snapshot: &SessionPresenceSnapshot) {
        let message = ServerMessage::SessionRoster {
            session_id: session_id.to_string(),
            version: snapshot.version,
            participant_ids: snapshot
                .participant_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
        };
        let mut connections = self.lock();
        for connection in connections.values_mut() {
            if connection.session_id != Some(session_id)
                || connection.roster_version == Some(snapshot.version)
            {
                continue;
            }
            match connection
                .sender
                .try_send(Outbound::Message(message.clone()))
            {
                Ok(()) => {
                    connection.roster_version = Some(snapshot.version);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.inner.metrics.send_dropped();
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {}
            }
        }
    }

    /// Enqueue a message for every connection in a session on this replica.
    ///
    /// Moderation uses this rather than the `_except` form because the whole
    /// roster needs to reflect the change, and the target in particular must
    /// learn that it was muted or removed. Delivery is replica-local.
    pub fn broadcast_to_session(&self, session_id: SessionId, message: &ServerMessage) {
        let targets: Vec<Uuid> = self
            .lock()
            .iter()
            .filter(|(_, connection)| connection.session_id == Some(session_id))
            .map(|(id, _)| *id)
            .collect();
        self.send_to(&targets, message);
    }

    fn send_to(&self, targets: &[Uuid], message: &ServerMessage) {
        if targets.is_empty() {
            return;
        }
        let connections = self.lock();
        for id in targets {
            if let Some(connection) = connections.get(id) {
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

    /// Enqueue a message for every connection in a session on this replica
    /// except the originating one.
    ///
    /// The originator already received a direct acknowledgement, and telling it
    /// that it joined when it *is* the participant is noise.
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
        self.send_to(&targets, message);
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
}

impl ConnectionRegistry {
    pub fn send_interaction_version(&self, session: SessionId, version: i64) {
        let message = crate::http::ws::envelope::ServerMessage::SessionInteractionsChanged {
            session_id: session.to_string(),
            version,
        };
        let mut connections = self.lock();
        for connection in connections.values_mut() {
            if connection.session_id == Some(session)
                && connection.interaction_version != Some(version)
            {
                match connection
                    .sender
                    .try_send(Outbound::Message(message.clone()))
                {
                    Ok(()) => connection.interaction_version = Some(version),
                    Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                        self.inner.metrics.send_dropped()
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {}
                }
            }
        }
    }
}
