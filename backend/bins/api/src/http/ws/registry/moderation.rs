use super::{ConnectionRegistry, Outbound};
use crate::http::ws::envelope::ServerMessage;
use mahfel_domain::{Participant, ParticipantId, SessionId};
use std::collections::HashMap;

impl ConnectionRegistry {
    pub(crate) fn reconcile_moderation(
        &self,
        session: SessionId,
        participants: &[Participant],
        live: bool,
    ) {
        let participants: HashMap<ParticipantId, &Participant> = participants
            .iter()
            .map(|participant| (participant.id, participant))
            .collect();
        for connection in self
            .lock()
            .values_mut()
            .filter(|connection| connection.session_id == Some(session))
        {
            if *connection.close.borrow() {
                continue;
            }
            let participant = connection
                .participant_id
                .and_then(|id| participants.get(&id));
            if !live || participant.is_none_or(|participant| !participant.is_active()) {
                connection.close_message = Some(ServerMessage::ParticipantRemoved {
                    session_id: session.to_string(),
                    participant_id: connection
                        .participant_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    actor_id: connection.actor_id.to_string(),
                    reason: if live { "removed" } else { "session_ended" }.to_owned(),
                });
                connection.close.send_replace(true);
                continue;
            }
            let Some(participant) = participant else {
                continue;
            };
            if connection.mute_kind == participant.mute_kind {
                continue;
            }
            let message = match participant.mute_kind {
                Some(kind) => ServerMessage::ParticipantMuted {
                    session_id: session.to_string(),
                    participant_id: participant.id.to_string(),
                    actor_id: participant.actor_id.to_string(),
                    kind: kind.as_str().to_owned(),
                    source: "microphone".to_owned(),
                },
                None => ServerMessage::ParticipantUnmuted {
                    session_id: session.to_string(),
                    participant_id: participant.id.to_string(),
                    actor_id: participant.actor_id.to_string(),
                    source: "microphone".to_owned(),
                },
            };
            if connection
                .sender
                .try_send(Outbound::Message(message))
                .is_ok()
            {
                connection.mute_kind = participant.mute_kind;
            } else {
                self.inner.metrics.send_dropped();
            }
        }
    }
}
