//! Session membership, per-actor generations, and replica-local presence.

use super::ConnectionRegistry;
use mahfel_domain::{ActorId, ParticipantId, RoomId, SessionId};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParticipationHandle {
    pub session_id: SessionId,
    pub room_id: RoomId,
    pub participant_id: ParticipantId,
    pub generation: u64,
}

impl ConnectionRegistry {
    /// Hand out the next generation for the connection's `(session, actor)`.
    pub fn joined(
        &self,
        id: Uuid,
        room_id: RoomId,
        session_id: SessionId,
        participant_id: ParticipantId,
    ) -> u64 {
        let generation = {
            let actor_id = self.actor_of(id).unwrap_or_default();
            let mut generations = self.generations();
            let entry = generations.entry((session_id, actor_id)).or_insert(0);
            *entry = entry.wrapping_add(1);
            *entry
        };
        if let Some(connection) = self.lock().get_mut(&id) {
            connection.session_id = Some(session_id);
            connection.room_id = Some(room_id);
            connection.participant_id = Some(participant_id);
            connection.roster_version = None;
            connection.generation = generation;
        }
        generation
    }

    pub(crate) fn left_if_current(&self, id: Uuid, expected: ParticipationHandle) -> bool {
        let mut connections = self.lock();
        let Some(connection) = connections.get_mut(&id) else {
            return false;
        };
        if connection.session_id != Some(expected.session_id)
            || connection.participant_id != Some(expected.participant_id)
            || connection.generation != expected.generation
        {
            return false;
        }
        connection.session_id = None;
        connection.room_id = None;
        connection.participant_id = None;
        connection.roster_version = None;
        true
    }

    #[must_use]
    pub(crate) fn participation(&self, id: Uuid) -> Option<ParticipationHandle> {
        self.lock().get(&id).and_then(|connection| {
            Some(ParticipationHandle {
                session_id: connection.session_id?,
                room_id: connection.room_id?,
                participant_id: connection.participant_id?,
                generation: connection.generation,
            })
        })
    }

    #[must_use]
    pub fn sessions_for_room(&self, room_id: RoomId) -> Vec<SessionId> {
        let mut sessions = Vec::new();
        for connection in self.lock().values() {
            if connection.room_id == Some(room_id)
                && let Some(session_id) = connection.session_id
                && !sessions.contains(&session_id)
            {
                sessions.push(session_id);
            }
        }
        sessions
    }

    #[must_use]
    pub fn joined_sessions(&self) -> Vec<SessionId> {
        let mut sessions = Vec::new();
        for connection in self.lock().values() {
            if let Some(session_id) = connection.session_id
                && !sessions.contains(&session_id)
            {
                sessions.push(session_id);
            }
        }
        sessions
    }

    /// How many connections this replica currently holds for a participant in a
    /// session.
    ///
    /// Presence is derived from this: a participant is present while at least
    /// one connection is live, so a participant on two devices stays present
    /// when one closes. A participant with devices on two replicas is counted
    /// once per replica.
    pub fn connection_count_for_participant(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> usize {
        self.lock()
            .values()
            .filter(|connection| {
                connection.session_id == Some(session_id) && connection.actor_id == actor_id
            })
            .count()
    }
}
