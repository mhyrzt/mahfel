use crate::http::ws::{
    envelope::{ChatEventPayload, ServerMessage},
    registry::ConnectionRegistry,
};
use mahfel_application::ports::{ChatActivitySnapshot, ChatActivityStore, ChatStore, FileStore};
use mahfel_domain::{ActorId, ChatEvent, ChatEventKind, FileContext, RoomId};
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
    files: Arc<dyn FileStore>,
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
        poll(chat, activity, files, registry, wake, shutdown).await;
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
    files: Arc<dyn FileStore>,
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
            let Some(allowed) = poll_room(
                chat.as_ref(),
                files.as_ref(),
                &registry,
                room_id,
                &subscribers,
            )
            .await
            else {
                continue;
            };
            if heartbeat_due {
                let mut actors = HashSet::new();
                for &(_, actor_id, _) in &subscribers {
                    if !actors.insert(actor_id) {
                        continue;
                    }
                    if !allowed.contains(&actor_id) {
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
    files: &dyn FileStore,
    registry: &ConnectionRegistry,
    room_id: RoomId,
    subscribers: &[(Uuid, ActorId, i64)],
) -> Option<HashSet<ActorId>> {
    let actors: Vec<_> = subscribers
        .iter()
        .map(|(_, actor, _)| *actor)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let mut allowed = HashSet::new();
    for batch in actors.chunks(1000) {
        match chat.authorized_subscribers(room_id, batch).await {
            Ok(members) => allowed.extend(members),
            Err(error) => {
                tracing::warn!(%error, "chat membership batch failed");
                return None;
            }
        }
    }
    for actor in &actors {
        if !allowed.contains(actor) {
            registry.unsubscribe_chat_actor(room_id, *actor);
        }
    }
    let minimum = subscribers
        .iter()
        .filter(|(_, actor, _)| allowed.contains(actor))
        .map(|(_, _, cursor)| *cursor)
        .min();
    let Some(minimum) = minimum else {
        return Some(allowed);
    };
    let actor = *allowed.iter().next()?;
    let events = match chat.list_events(room_id, actor, minimum, 100).await {
        Ok(events) => events,
        Err(error) => {
            tracing::warn!(%error, "chat event catch-up failed");
            return None;
        }
    };
    let ids: Vec<_> = events.iter().map(|event| event.message.id).collect();
    let ready_files = match files.list_for_messages(&ids).await {
        Ok(files) => files,
        Err(error) => {
            tracing::warn!(%error, "chat attachment batch failed");
            return None;
        }
    };
    let mut attachments = HashMap::<_, Vec<String>>::new();
    for file in ready_files {
        if let FileContext::ChatMessage(id) = file.context {
            attachments.entry(id).or_default().push(file.id.to_string());
        }
    }
    for &(id, actor_id, cursor) in subscribers {
        if !allowed.contains(&actor_id) {
            continue;
        }
        for event in events.iter().filter(|event| event.sequence > cursor) {
            let message = match frame(
                event,
                attachments
                    .get(&event.message.id)
                    .cloned()
                    .unwrap_or_default(),
            ) {
                Ok(message) => message,
                Err(error) => {
                    tracing::warn!(%error, "chat event timestamp cannot be encoded");
                    return None;
                }
            };
            registry.send_chat_event(id, room_id, event.sequence, message);
        }
    }
    Some(allowed)
}

fn frame(
    event: &ChatEvent,
    attachment_ids: Vec<String>,
) -> Result<ServerMessage, time::error::Format> {
    let kind = match event.kind {
        ChatEventKind::Created => "created",
        ChatEventKind::Edited => "edited",
        ChatEventKind::Deleted => "deleted",
        ChatEventKind::ReactionSet => "reaction_set",
        ChatEventKind::ReactionRemoved => "reaction_removed",
    };
    Ok(ServerMessage::ChatEvent {
        event: Box::new(ChatEventPayload {
            room_id: event.room_id.to_string(),
            sequence: event.sequence,
            event_id: event.id.to_string(),
            kind: kind.to_owned(),
            message_id: event.message.id.to_string(),
            parent_id: event.message.parent_id.map(|id| id.to_string()),
            session_id: event.message.session_id.map(|id| id.to_string()),
            author_id: event.message.author_id.to_string(),
            reaction_actor_id: event.reaction_actor_id.map(|id| id.to_string()),
            reaction_emoji: event.reaction_emoji.clone(),
            content: event
                .message
                .deleted_at
                .is_none()
                .then(|| event.message.content.clone()),
            created_at: event
                .message
                .created_at
                .format(&time::format_description::well_known::Rfc3339)?,
            edited_at: event
                .message
                .edited_at
                .map(|time| time.format(&time::format_description::well_known::Rfc3339))
                .transpose()?,
            deleted_at: event
                .message
                .deleted_at
                .map(|time| time.format(&time::format_description::well_known::Rfc3339))
                .transpose()?,
            attachment_ids,
        }),
    })
}
