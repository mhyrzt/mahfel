//! Resolve media actor IDs to authorized live-session participants.

use super::SpeakerProjection;
use crate::http::ws::{
    envelope::ServerMessage, placement::MediaPlacementRouter, registry::ConnectionRegistry,
};
use mahfel_application::ports::{ParticipantStore, SessionStore};
use mahfel_domain::{ActorId, NodeId, RoomId, SessionId, SessionStatus};
use mahfel_rtc_protocol::media::WatchActiveSpeakersResponse;
use uuid::Uuid;

impl SpeakerProjection {
    pub(crate) async fn apply_from_node(
        &self,
        frame: WatchActiveSpeakersResponse,
        node_id: &NodeId,
        router: &MediaPlacementRouter,
        registry: &ConnectionRegistry,
        participants: &dyn ParticipantStore,
        sessions: &dyn SessionStore,
    ) {
        if frame.reset {
            self.clear_for_node(node_id, router, registry, sessions)
                .await;
            return;
        }
        let Ok(media_room_uuid) = Uuid::parse_str(&frame.room_id) else {
            return;
        };
        let session_id = SessionId::from_uuid(media_room_uuid);
        let Ok(Some(session)) = sessions.get(session_id).await else {
            return;
        };
        if session.status != SessionStatus::Live
            || !registry
                .sessions_for_room(session.room_id)
                .contains(&session_id)
            || !matches!(router.is_assigned(session_id, node_id).await, Ok(true))
        {
            return;
        }
        self.apply(
            WatchActiveSpeakersResponse {
                room_id: session.room_id.to_string(),
                actor_ids: frame.actor_ids,
                reset: false,
            },
            registry,
            participants,
            sessions,
        )
        .await;
    }

    pub(crate) async fn clear_for_node(
        &self,
        node_id: &NodeId,
        router: &MediaPlacementRouter,
        registry: &ConnectionRegistry,
        sessions: &dyn SessionStore,
    ) {
        for session_id in registry.joined_sessions() {
            if !matches!(router.is_assigned(session_id, node_id).await, Ok(true)) {
                continue;
            }
            if let Ok(Some(session)) = sessions.get(session_id).await {
                self.lock().rooms.remove(&session.room_id);
            }
            self.clear_session(session_id, registry);
        }
    }

    pub(crate) async fn apply(
        &self,
        frame: WatchActiveSpeakersResponse,
        registry: &ConnectionRegistry,
        participants: &dyn ParticipantStore,
        sessions: &dyn SessionStore,
    ) {
        if frame.reset {
            self.clear(registry);
            return;
        }
        let Ok(room_uuid) = Uuid::parse_str(&frame.room_id) else {
            tracing::warn!("media sent an invalid speaker room id");
            return;
        };
        let room_id = RoomId::from_uuid(room_uuid);
        let actors: Vec<_> = frame
            .actor_ids
            .iter()
            .take(8)
            .filter_map(|raw| Uuid::parse_str(raw).ok().map(ActorId::from_uuid))
            .collect();
        {
            let mut state = self.lock();
            if actors.is_empty() {
                state.rooms.remove(&room_id);
            } else {
                state.rooms.insert(room_id, actors.clone());
            }
        }
        for session_id in registry.sessions_for_room(room_id) {
            let session = match sessions.get(session_id).await {
                Ok(Some(session)) => session,
                Ok(None) => {
                    self.clear_session(session_id, registry);
                    continue;
                }
                Err(error) => {
                    tracing::warn!(?error, "speaker session lookup failed");
                    self.clear_session(session_id, registry);
                    continue;
                }
            };
            if session.room_id != room_id || session.status != SessionStatus::Live {
                self.clear_session(session_id, registry);
                continue;
            }
            let mut resolved = Vec::with_capacity(actors.len());
            for actor_id in &actors {
                let participant_id = match participants.get_active(session_id, *actor_id).await {
                    Ok(Some(participant)) => Some(participant.id),
                    Ok(None) => None,
                    Err(error) => {
                        tracing::warn!(?error, "speaker participant lookup failed");
                        None
                    }
                };
                if let Some(id) = participant_id {
                    resolved.push(id);
                }
            }
            let changed = {
                let mut state = self.lock();
                if state.sessions.get(&session_id) == Some(&resolved)
                    || (resolved.is_empty() && !state.sessions.contains_key(&session_id))
                {
                    false
                } else {
                    state.sessions.insert(session_id, resolved.clone());
                    true
                }
            };
            if changed {
                registry.broadcast_to_session(
                    session_id,
                    &ServerMessage::ActiveSpeakers {
                        session_id: session_id.to_string(),
                        participant_ids: resolved.iter().map(ToString::to_string).collect(),
                    },
                );
            }
        }
    }

    pub async fn replay_room(
        &self,
        room_id: RoomId,
        registry: &ConnectionRegistry,
        participants: &dyn ParticipantStore,
        sessions: &dyn SessionStore,
    ) {
        let actors = self.lock().rooms.get(&room_id).cloned().unwrap_or_default();
        self.apply(
            WatchActiveSpeakersResponse {
                room_id: room_id.to_string(),
                actor_ids: actors.iter().map(ToString::to_string).collect(),
                reset: false,
            },
            registry,
            participants,
            sessions,
        )
        .await;
    }
}
