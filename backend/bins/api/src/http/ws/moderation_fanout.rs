use super::{envelope::ServerMessage, state::WsState};
use mahfel_application::ports::CoordinationStore;
use mahfel_domain::{RoomStatus, SessionId, SessionStatus};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Notice {
    origin: Uuid,
    session: SessionId,
    message: ServerMessage,
}

pub(super) async fn publish(state: &WsState, session: SessionId, message: ServerMessage) {
    state.registry.broadcast_to_session(session, &message);
    if let Some(redis) = &state.session_presence {
        let notice = Notice {
            origin: state.moderation_origin,
            session,
            message,
        };
        if let Ok(payload) = serde_json::to_vec(&notice)
            && let Err(error) = redis
                .publish(&redis.keys().channel("session-moderation"), &payload)
                .await
        {
            tracing::warn!(%error, "moderation notice lost; durable reconciliation will recover");
        }
    }
}

pub(super) fn spawn(state: WsState) {
    let receiver = state.clone();
    tokio::spawn(async move {
        let Some(redis) = &receiver.session_presence else {
            return;
        };
        let mut shutdown = receiver.shutdown.clone();
        loop {
            if *shutdown.borrow() {
                return;
            }
            match redis
                .subscribe(&redis.keys().channel("session-moderation"))
                .await
            {
                Ok(mut messages) => loop {
                    tokio::select! {
                        _ = shutdown.changed() => return,
                        payload = messages.recv() => {
                            let Some(payload) = payload else { break; };
                            if let Ok(notice) = serde_json::from_slice::<Notice>(&payload.payload)
                                && notice.origin != receiver.moderation_origin {
                                receiver.registry.broadcast_to_session(notice.session, &notice.message);
                            }
                        }
                    }
                },
                Err(error) => tracing::warn!(%error, "moderation subscriber unavailable"),
            }
            tokio::select! { _ = shutdown.changed() => return, _ = tokio::time::sleep(Duration::from_secs(2)) => {} }
        }
    });
    tokio::spawn(async move {
        let mut shutdown = state.shutdown.clone();
        let mut poll = tokio::time::interval(Duration::from_secs(2));
        poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! { _ = shutdown.changed() => return, _ = poll.tick() => {} }
            for session_id in state.registry.joined_sessions() {
                let session = match state.stores.sessions.get(session_id).await {
                    Ok(Some(session)) => session,
                    Ok(None) => {
                        state.registry.reconcile_moderation(session_id, &[], false);
                        continue;
                    }
                    Err(error) => {
                        tracing::warn!(?error, "moderation session reconciliation unavailable");
                        continue;
                    }
                };
                let room = match state.stores.rooms.get(session.room_id).await {
                    Ok(room) => room,
                    Err(error) => {
                        tracing::warn!(?error, "moderation room reconciliation unavailable");
                        continue;
                    }
                };
                match state.stores.participants.list_for_session(session_id).await {
                    Ok(participants) => state.registry.reconcile_moderation(
                        session_id,
                        &participants,
                        session.status == SessionStatus::Live
                            && room.is_some_and(|room| room.status == RoomStatus::Active),
                    ),
                    Err(error) => {
                        tracing::warn!(?error, "moderation participant reconciliation unavailable")
                    }
                }
            }
        }
    });
}
