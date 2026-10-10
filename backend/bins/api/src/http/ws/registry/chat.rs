use crate::http::ws::{
    envelope::ServerMessage,
    registry::{ConnectionRegistry, Outbound},
};
use mahfel_domain::{ActorId, RoomId};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

impl ConnectionRegistry {
    pub fn allow_typing_update(&self, id: Uuid, room_id: RoomId) -> bool {
        let mut connections = self.lock();
        let Some(connection) = connections.get_mut(&id) else {
            return false;
        };
        if !connection.chat_subscriptions.contains_key(&room_id) {
            return false;
        }
        let now = Instant::now();
        if connection
            .last_typing
            .get(&room_id)
            .is_some_and(|last| now.duration_since(*last) < Duration::from_millis(500))
        {
            return false;
        }
        connection.last_typing.insert(room_id, now);
        true
    }

    pub fn is_subscribed_chat(&self, id: Uuid, room_id: RoomId) -> bool {
        self.lock()
            .get(&id)
            .is_some_and(|connection| connection.chat_subscriptions.contains_key(&room_id))
    }

    pub fn broadcast_chat_activity(&self, room_id: RoomId, message: ServerMessage) {
        let mut connections = self.lock();
        for connection in connections.values_mut() {
            if !connection.chat_subscriptions.contains_key(&room_id) {
                continue;
            }
            if let Err(error) = connection
                .sender
                .try_send(Outbound::Message(message.clone()))
            {
                if matches!(error, mpsc::error::TrySendError::Full(_)) {
                    self.inner.metrics.send_dropped();
                }
                connection.chat_subscriptions.clear();
                connection.last_typing.clear();
                connection.close.send_replace(true);
            }
        }
    }

    pub fn subscribe_chat(&self, id: Uuid, room_id: RoomId, after_sequence: i64) -> bool {
        let mut connections = self.lock();
        let Some(connection) = connections.get_mut(&id) else {
            return false;
        };
        if !connection.chat_subscriptions.contains_key(&room_id)
            && connection.chat_subscriptions.len() >= 16
        {
            return false;
        }
        connection
            .chat_subscriptions
            .entry(room_id)
            .or_insert(after_sequence);
        true
    }

    pub fn unsubscribe_chat(&self, id: Uuid, room_id: RoomId) {
        if let Some(connection) = self.lock().get_mut(&id) {
            connection.chat_subscriptions.remove(&room_id);
            connection.last_typing.remove(&room_id);
        }
    }

    pub fn unsubscribe_chat_actor(&self, room_id: RoomId, actor_id: ActorId) {
        for connection in self.lock().values_mut() {
            if connection.actor_id == actor_id {
                connection.chat_subscriptions.remove(&room_id);
                connection.last_typing.remove(&room_id);
            }
        }
    }

    /// A snapshot avoids holding the registry mutex across database calls.
    pub fn chat_subscriptions(&self) -> HashMap<RoomId, Vec<(Uuid, ActorId, i64)>> {
        let mut rooms: HashMap<RoomId, Vec<(Uuid, ActorId, i64)>> = HashMap::new();
        for (id, connection) in self.lock().iter() {
            for (&room_id, &sequence) in &connection.chat_subscriptions {
                rooms
                    .entry(room_id)
                    .or_default()
                    .push((*id, connection.actor_id, sequence));
            }
        }
        rooms
    }

    /// Advance only after the event is queued. If a client is too slow, drop
    /// its connection so it must reconnect and replay from its received cursor.
    pub fn send_chat_event(
        &self,
        id: Uuid,
        room_id: RoomId,
        sequence: i64,
        message: ServerMessage,
    ) {
        let mut connections = self.lock();
        let Some(connection) = connections.get_mut(&id) else {
            return;
        };
        let Some(cursor) = connection.chat_subscriptions.get_mut(&room_id) else {
            return;
        };
        if sequence <= *cursor {
            return;
        }
        match connection.sender.try_send(Outbound::Message(message)) {
            Ok(()) => *cursor = sequence,
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.inner.metrics.send_dropped();
                connection.chat_subscriptions.clear();
                connection.last_typing.clear();
                connection.close.send_replace(true);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                connection.chat_subscriptions.clear();
                connection.last_typing.clear();
                connection.close.send_replace(true);
            }
        }
    }
}
