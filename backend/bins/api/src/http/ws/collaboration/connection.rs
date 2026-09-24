use super::{
    Fanout,
    document::{DocumentState, apply},
};
use crate::http::ws::tickets;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::StreamExt;
use mahfel_application::ports::{CollaborationStore, NewCollaborationEvent};
use mahfel_domain::{CollaborationEventKind, CollaborationKind, SessionId};
use mahfel_redis::RedisCoordination;
use serde::Deserialize;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::time::{Instant, MissedTickBehavior, interval, timeout};
use uuid::Uuid;
use yrs::{
    Update,
    sync::{Message, SyncMessage},
    updates::{decoder::Decode, encoder::Encode},
};

const MAX_AWARENESS_BYTES: usize = 4096;
const MAX_FRAMES_PER_SECOND: u32 = 100;

#[derive(Deserialize)]
struct AuthFrame {
    #[serde(rename = "type")]
    kind: String,
    ticket: String,
}

pub(super) async fn run(
    mut socket: WebSocket,
    session_id: SessionId,
    kind: CollaborationKind,
    store: Arc<dyn CollaborationStore>,
    fanout: Arc<Fanout>,
    tickets_store: RedisCoordination,
) {
    let principal = match timeout(Duration::from_secs(10), socket.next()).await {
        Ok(Some(Ok(WsMessage::Text(text)))) => {
            let Ok(frame) = serde_json::from_str::<AuthFrame>(&text) else {
                return;
            };
            if frame.kind != "auth" {
                return;
            }
            let Some(principal) = tickets::consume(Some(&tickets_store), &frame.ticket).await
            else {
                return;
            };
            principal
        }
        _ => return,
    };
    let actor = principal.actor_id;
    let Ok(access) = store.access(session_id, actor).await else {
        return;
    };
    if !access.can_read {
        return;
    }
    let Ok(Some(document)) = store.document(session_id, kind).await else {
        return;
    };
    if document.schema_version != CollaborationKind::SCHEMA_VERSION {
        return;
    }
    let mut state = match DocumentState::load(store.as_ref(), session_id, actor, document.id).await
    {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, document_id = %document.id, "collaboration document load failed");
            return;
        }
    };
    if send_json(
        &mut socket,
        json!({
            "type": "collaboration_ready", "document_id": document.id,
            "schema_version": document.schema_version, "can_edit": access.can_edit,
            "live": access.live, "sequence": state.cursor,
        }),
    )
    .await
    .is_err()
    {
        return;
    }
    if send_binary(
        &mut socket,
        Message::Sync(SyncMessage::SyncStep1(state.vector())),
    )
    .await
    .is_err()
    {
        return;
    }
    let Ok(viewports) = store.latest_viewports(session_id, actor).await else {
        return;
    };
    for event in viewports {
        if send_json(
            &mut socket,
            json!({
                "type":"collaboration_event", "sequence":event.sequence,
                "kind":event.kind.as_str(), "data":event.data,
            }),
        )
        .await
        .is_err()
        {
            return;
        }
    }
    let mut awareness = fanout.subscribe(document.id.as_uuid());
    let mut poll = interval(Duration::from_millis(500));
    poll.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_edit = access.can_edit;
    let mut rate_window = Instant::now();
    let mut rate_count = 0_u32;

    loop {
        tokio::select! {
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(WsMessage::Binary(bytes))) => {
                        if rate_window.elapsed() >= Duration::from_secs(1) {
                            rate_window = Instant::now();
                            rate_count = 0;
                        }
                        rate_count += 1;
                        if rate_count > MAX_FRAMES_PER_SECOND { return; }
                        let Ok(message) = Message::decode_v1(&bytes) else { return; };
                        match message {
                            Message::Sync(SyncMessage::SyncStep1(vector)) => {
                                if state.refresh(store.as_ref(), session_id, actor).await.is_err() { return; }
                                if send_binary(&mut socket, Message::Sync(SyncMessage::SyncStep2(state.diff(&vector)))).await.is_err() { return; }
                                if send_binary(&mut socket, Message::Sync(SyncMessage::SyncStep1(state.vector()))).await.is_err() { return; }
                            }
                            Message::Sync(SyncMessage::SyncStep2(update))
                            | Message::Sync(SyncMessage::Update(update)) => {
                                if Update::decode_v1(&update).is_err() { return; }
                                let Ok(access) = store.access(session_id, actor).await else { return; };
                                if !access.can_edit || !access.live {
                                    if send_json(&mut socket, json!({"type":"permission_changed","can_edit":false})).await.is_err() { return; }
                                    continue;
                                }
                                let event = NewCollaborationEvent {
                                    session_id, id: Uuid::new_v4(), actor_id: actor,
                                    document_id: Some(document.id), kind: CollaborationEventKind::DocumentUpdate,
                                    update: Some(update.clone()), data: json!({}),
                                };
                                if apply(&state.doc, &update).is_err() { return; }
                                if store.append(&event).await.is_err() { return; }
                            }
                            Message::Awareness(_) => {
                                if bytes.len() > MAX_AWARENESS_BYTES { return; }
                                fanout.publish(document.id.as_uuid(), bytes.to_vec()).await;
                            }
                            Message::AwarenessQuery => {}
                            Message::Auth(_) | Message::Custom(_, _) => return,
                        }
                    }
                    Some(Ok(WsMessage::Ping(bytes))) => {
                        if socket.send(WsMessage::Pong(bytes)).await.is_err() { return; }
                    }
                    Some(Ok(WsMessage::Pong(_))) => {}
                    Some(Ok(WsMessage::Text(_))) => return,
                    Some(Ok(WsMessage::Close(_))) | Some(Err(_)) | None => return,
                }
            }
            notice = awareness.recv() => {
                match notice {
                    Ok(notice) => {
                        if socket.send(WsMessage::Binary(notice.payload.into())).await.is_err() { return; }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => return,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
            _ = poll.tick() => {
                let Ok(events) = state.refresh(store.as_ref(), session_id, actor).await else { return; };
                for event in events {
                    if event.kind == CollaborationEventKind::DocumentUpdate {
                        if event.document_id == Some(document.id)
                            && let Some(update) = event.update
                            && send_binary(&mut socket, Message::Sync(SyncMessage::Update(update))).await.is_err()
                        { return; }
                    } else if send_json(&mut socket, json!({
                        "type":"collaboration_event", "sequence":event.sequence,
                        "kind":event.kind.as_str(), "data":event.data,
                    })).await.is_err() { return; }
                }
                let Ok(access) = store.access(session_id, actor).await else { return; };
                if !access.can_read { return; }
                if access.can_edit != last_edit {
                    last_edit = access.can_edit;
                    if send_json(&mut socket, json!({"type":"permission_changed","can_edit":last_edit})).await.is_err() { return; }
                }
            }
        }
    }
}

async fn send_binary(socket: &mut WebSocket, message: Message) -> Result<(), axum::Error> {
    socket
        .send(WsMessage::Binary(message.encode_v1().into()))
        .await
}

async fn send_json(socket: &mut WebSocket, value: serde_json::Value) -> Result<(), axum::Error> {
    socket.send(WsMessage::Text(value.to_string().into())).await
}
