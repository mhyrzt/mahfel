use crate::http::ws::{
    envelope::{ChatEventPayload, ServerMessage},
    registry::ConnectionRegistry,
};
use mahfel_application::ports::{ChatActivitySnapshot, ChatActivityStore, ChatStore};
use mahfel_domain::{ActorId, ChatEvent, ChatEventKind, RoomId};
use mahfel_redis::RedisCoordination;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Notify, watch},
    time::MissedTickBehavior,
};
use uuid::Uuid;

/// Redis only accelerates delivery. The database event log remains the replay
/// source when a publish is lost or a replica restarts.
pub(super) fn spawn(
    chat: Arc<dyn ChatStore>,
    activity: Arc<dyn ChatActivityStore>,
    redis: RedisCoordination,
    registry: ConnectionRegistry,
    wake: Arc<Notify>,
    shutdown: watch::Receiver<bool>,
) {
    let listener_wake = Arc::clone(&wake);
    let listener_shutdown = shutdown.clone();
    tokio::spawn(async move {
        listen(redis, listener_wake, listener_shutdown).await;
    });
    tokio::spawn(async move {
        poll(chat, activity, registry, wake, shutdown).await;
    });
}

async fn listen(redis: RedisCoordination, wake: Arc<Notify>, mut shutdown: watch::Receiver<bool>) {
    let channel = redis.keys().channel("chat-events");
    loop {
        if *shutdown.borrow() {
            return;
        }
        match redis.subscribe(&channel).await {
            Ok(mut messages) => loop {
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { return; }
                    }
                    message = messages.recv() => {
                        if message.is_none() { break; }
                        wake.notify_one();
                    }
                }
            },
            Err(error) => tracing::warn!(%error, "chat wakeup subscription failed"),
        }
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
            () = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}

async fn poll(
    chat: Arc<dyn ChatStore>,
    activity: Arc<dyn ChatActivityStore>,
    registry: ConnectionRegistry,
    wake: Arc<Notify>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_heartbeat = Instant::now() - Duration::from_secs(15);
    let mut last_snapshots: HashMap<RoomId, ChatActivitySnapshot> = HashMap::new();
    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
            _ = interval.tick() => {}
            () = wake.notified() => {}
        }
        let rooms = registry.chat_subscriptions();
        let heartbeat_due = last_heartbeat.elapsed() >= Duration::from_secs(15);
        last_snapshots.retain(|room_id, _| rooms.contains_key(room_id));
        for (room_id, subscribers) in rooms {
            poll_room(chat.as_ref(), &registry, room_id, &subscribers).await;
            if heartbeat_due {
                let mut actors = HashSet::new();
                for &(_, actor_id, _) in &subscribers {
                    if !actors.insert(actor_id) {
                        continue;
                    }
                    if chat.event_head(room_id, actor_id).await.is_err() {
                        registry.unsubscribe_chat_actor(room_id, actor_id);
                        continue;
                    }
                    if let Err(error) = activity
                        .heartbeat(room_id, actor_id, Duration::from_secs(45))
                        .await
                    {
                        tracing::warn!(%error, "chat presence heartbeat failed");
                    }
                }
            }
            match activity.snapshot(room_id).await {
                Ok(snapshot) if last_snapshots.get(&room_id) != Some(&snapshot) => {
                    registry.broadcast_chat_activity(
                        room_id,
                        ServerMessage::ChatActivity {
                            room_id: room_id.to_string(),
                            online_count: snapshot.online_count,
                            typing_count: snapshot.typing_count,
                        },
                    );
                    last_snapshots.insert(room_id, snapshot);
                }
                Ok(_) => {}
                Err(error) => tracing::warn!(%error, "chat activity snapshot failed"),
            }
        }
        if heartbeat_due {
            last_heartbeat = Instant::now();
        }
    }
}

async fn poll_room(
    chat: &dyn ChatStore,
    registry: &ConnectionRegistry,
    room_id: RoomId,
    subscribers: &[(Uuid, ActorId, i64)],
) {
    let Some(minimum) = subscribers.iter().map(|(_, _, cursor)| *cursor).min() else {
        return;
    };
    let mut events = None;
    for &(_, actor_id, _) in subscribers {
        match chat.list_events(room_id, actor_id, minimum, 100).await {
            Ok(batch) => {
                events = Some(batch);
                break;
            }
            Err(mahfel_application::ports::StoreError::NotFound(_)) => {
                registry.unsubscribe_chat_actor(room_id, actor_id);
            }
            Err(error) => {
                tracing::warn!(%error, "chat event catch-up failed");
                return;
            }
        }
    }
    let Some(events) = events else {
        return;
    };
    if events.is_empty() {
        return;
    }
    for &(id, actor_id, cursor) in subscribers {
        if events.iter().all(|event| event.sequence <= cursor) {
            continue;
        }
        if chat.event_head(room_id, actor_id).await.is_err() {
            registry.unsubscribe_chat_actor(room_id, actor_id);
            continue;
        }
        for event in events.iter().filter(|event| event.sequence > cursor) {
            registry.send_chat_event(id, room_id, event.sequence, frame(event));
        }
    }
}

fn frame(event: &ChatEvent) -> ServerMessage {
    let kind = match event.kind {
        ChatEventKind::Created => "created",
        ChatEventKind::Edited => "edited",
        ChatEventKind::Deleted => "deleted",
        ChatEventKind::ReactionSet => "reaction_set",
        ChatEventKind::ReactionRemoved => "reaction_removed",
    };
    ServerMessage::ChatEvent {
        event: Box::new(ChatEventPayload {
            room_id: event.room_id.to_string(),
            sequence: event.sequence,
            event_id: event.id.to_string(),
            kind: kind.to_owned(),
            message_id: event.message.id.to_string(),
            parent_id: event.message.parent_id.map(|id| id.to_string()),
            author_id: event.message.author_id.to_string(),
            reaction_actor_id: event.reaction_actor_id.map(|id| id.to_string()),
            reaction_emoji: event.reaction_emoji.clone(),
            content: event
                .message
                .deleted_at
                .is_none()
                .then(|| event.message.content.clone()),
            created_at: event.message.created_at.to_string(),
            edited_at: event.message.edited_at.map(|time| time.to_string()),
            deleted_at: event.message.deleted_at.map(|time| time.to_string()),
        }),
    }
}
