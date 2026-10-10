//! Project trusted media snapshots onto locally joined session connections.

mod projection;
mod stream;

use super::{envelope::ServerMessage, registry::ConnectionRegistry};
use mahfel_domain::{ActorId, ParticipantId, RoomId, SessionId};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

#[derive(Clone, Default)]
pub struct SpeakerProjection {
    inner: Arc<Mutex<ProjectionState>>,
}

#[derive(Default)]
struct ProjectionState {
    rooms: HashMap<RoomId, Vec<ActorId>>,
    sessions: HashMap<SessionId, Vec<ParticipantId>>,
}

impl SpeakerProjection {
    #[must_use]
    pub fn current(&self, session_id: SessionId) -> Vec<ParticipantId> {
        self.lock()
            .sessions
            .get(&session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn clear_session(&self, session_id: SessionId, registry: &ConnectionRegistry) {
        let was_nonempty = self
            .lock()
            .sessions
            .remove(&session_id)
            .is_some_and(|speakers| !speakers.is_empty());
        if was_nonempty {
            registry.broadcast_to_session(
                session_id,
                &ServerMessage::ActiveSpeakers {
                    session_id: session_id.to_string(),
                    participant_ids: Vec::new(),
                },
            );
        }
    }

    pub fn prune(&self, registry: &ConnectionRegistry) {
        let joined = registry.joined_sessions();
        let rooms = registry.joined_rooms();
        let mut state = self.lock();
        state.sessions.retain(|id, _| joined.contains(id));
        state.rooms.retain(|id, _| rooms.contains(id));
    }

    fn clear(&self, registry: &ConnectionRegistry) {
        let mut state = self.lock();
        let sessions: Vec<_> = state
            .sessions
            .iter()
            .filter(|(_, speakers)| !speakers.is_empty())
            .map(|(id, _)| *id)
            .collect();
        state.rooms.clear();
        state.sessions.clear();
        drop(state);
        for session_id in sessions {
            registry.broadcast_to_session(
                session_id,
                &ServerMessage::ActiveSpeakers {
                    session_id: session_id.to_string(),
                    participant_ids: Vec::new(),
                },
            );
        }
    }

    fn lock(&self) -> MutexGuard<'_, ProjectionState> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
