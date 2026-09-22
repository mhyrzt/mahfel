//! Ephemeral audio-level aggregation. The observer callback never awaits.

#[cfg(test)]
mod tests;
mod tracker;

use self::tracker::RoomSpeakers;
use mahfel_domain::{ActorId, RoomId};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Instant,
};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeakerSnapshot {
    pub room_id: RoomId,
    pub actor_ids: Vec<ActorId>,
}

#[derive(Clone)]
pub struct SpeakerHub {
    state: Arc<Mutex<HashMap<RoomId, RoomSpeakers>>>,
    sender: broadcast::Sender<SpeakerSnapshot>,
}

impl SpeakerHub {
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(128);
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            sender,
        }
    }

    /// Subscription and initial state share the state lock with publishers.
    /// Changes therefore cannot overtake the initial snapshots.
    pub fn subscribe_with_snapshots(
        &self,
    ) -> (broadcast::Receiver<SpeakerSnapshot>, Vec<SpeakerSnapshot>) {
        let rooms = self.lock();
        let receiver = self.sender.subscribe();
        let snapshots = rooms
            .iter()
            .map(|(room_id, room)| SpeakerSnapshot {
                room_id: *room_id,
                actor_ids: room.published().to_vec(),
            })
            .collect();
        (receiver, snapshots)
    }

    #[must_use]
    pub fn active_count(&self) -> usize {
        self.lock()
            .values()
            .map(|room| room.published().len())
            .sum()
    }

    pub fn add_room(&self, room_id: RoomId) {
        self.lock().entry(room_id).or_default();
    }

    pub fn remove_room(&self, room_id: RoomId) {
        let mut rooms = self.lock();
        let removed = rooms.remove(&room_id);
        if removed.is_some() {
            let _ = self.sender.send(SpeakerSnapshot {
                room_id,
                actor_ids: Vec::new(),
            });
        }
    }

    pub fn clear_room(&self, room_id: RoomId) {
        let mut rooms = self.lock();
        if let Some(room) = rooms.get_mut(&room_id) {
            *room = RoomSpeakers::default();
            let _ = self.sender.send(SpeakerSnapshot {
                room_id,
                actor_ids: Vec::new(),
            });
        }
    }

    pub fn add_producer(&self, room_id: RoomId, producer_id: String, actor_id: ActorId) {
        self.lock()
            .entry(room_id)
            .or_default()
            .add_producer(producer_id, actor_id);
    }

    pub fn remove_producer(&self, room_id: RoomId, producer_id: &str) {
        let mut rooms = self.lock();
        let Some(room) = rooms.get_mut(&room_id) else {
            return;
        };
        if !room.remove_producer(producer_id) {
            return;
        }
        let snapshot = room.changed_snapshot(room_id, Instant::now(), true);
        if let Some(snapshot) = snapshot {
            let _ = self.sender.send(snapshot);
        }
    }

    /// Process one observer interval. Unknown producers are ignored.
    pub fn observe(&self, room_id: RoomId, volumes: &[(String, i8)]) {
        let mut rooms = self.lock();
        let Some(room) = rooms.get_mut(&room_id) else {
            return;
        };
        room.observe(volumes);
        let snapshot = room.changed_snapshot(room_id, Instant::now(), false);
        if let Some(snapshot) = snapshot {
            let _ = self.sender.send(snapshot);
        }
    }

    /// The mediasoup silence callback fires on transition, not every interval.
    pub fn silence(&self, room_id: RoomId) {
        let mut rooms = self.lock();
        let Some(room) = rooms.get_mut(&room_id) else {
            return;
        };
        room.silence();
        let snapshot = room.changed_snapshot(room_id, Instant::now(), true);
        if let Some(snapshot) = snapshot {
            let _ = self.sender.send(snapshot);
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<RoomId, RoomSpeakers>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for SpeakerHub {
    fn default() -> Self {
        Self::new()
    }
}
