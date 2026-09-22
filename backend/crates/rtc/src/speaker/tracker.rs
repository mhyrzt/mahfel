//! Per-room audio-level smoothing, hysteresis, and snapshot rate limiting.

use super::SpeakerSnapshot;
use mahfel_domain::{ActorId, RoomId};
use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

const ENTER_DBVO: f32 = -65.0;
const EXIT_DBVO: f32 = -75.0;
const MAX_SPEAKERS: usize = 8;
pub(super) const MIN_EMIT_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Default)]
pub(super) struct RoomSpeakers {
    owners: HashMap<String, ActorId>,
    levels: HashMap<ActorId, Level>,
    published: Vec<ActorId>,
    last_emitted: Option<Instant>,
}

#[derive(Clone, Copy)]
struct Level {
    smoothed: f32,
    above: u8,
    below: u8,
    active: bool,
}

impl RoomSpeakers {
    pub(super) fn published(&self) -> &[ActorId] {
        &self.published
    }

    pub(super) fn add_producer(&mut self, producer_id: String, actor_id: ActorId) {
        self.owners.insert(producer_id, actor_id);
    }

    pub(super) fn remove_producer(&mut self, producer_id: &str) -> bool {
        let Some(actor_id) = self.owners.remove(producer_id) else {
            return false;
        };
        if !self.owners.values().any(|owner| *owner == actor_id) {
            self.levels.remove(&actor_id);
        }
        true
    }

    pub(super) fn silence(&mut self) {
        for level in self.levels.values_mut() {
            level.active = false;
            level.above = 0;
            level.below = 3;
        }
    }

    pub(super) fn observe(&mut self, volumes: &[(String, i8)]) {
        let mut per_actor: HashMap<ActorId, i8> = HashMap::new();
        for (producer_id, volume) in volumes {
            if let Some(actor) = self.owners.get(producer_id) {
                per_actor
                    .entry(*actor)
                    .and_modify(|current| *current = (*current).max(*volume))
                    .or_insert(*volume);
            }
        }
        for (actor, volume) in &per_actor {
            let level = self.levels.entry(*actor).or_insert(Level {
                smoothed: f32::from(*volume),
                above: 0,
                below: 0,
                active: false,
            });
            level.smoothed = level.smoothed * 0.65 + f32::from(*volume) * 0.35;
            if level.smoothed >= ENTER_DBVO {
                level.above = level.above.saturating_add(1);
            } else {
                level.above = 0;
            }
            if level.smoothed <= EXIT_DBVO {
                level.below = level.below.saturating_add(1);
            } else {
                level.below = 0;
            }
            if level.above >= 2 {
                level.active = true;
            }
            if level.below >= 3 {
                level.active = false;
            }
        }
        for (actor, level) in &mut self.levels {
            if !per_actor.contains_key(actor) {
                level.above = 0;
                level.below = level.below.saturating_add(1);
                if level.below >= 3 {
                    level.active = false;
                }
            }
        }
    }

    pub(super) fn changed_snapshot(
        &mut self,
        room_id: RoomId,
        now: Instant,
        force: bool,
    ) -> Option<SpeakerSnapshot> {
        let mut actors: Vec<_> = self
            .levels
            .iter()
            .filter(|(_, level)| level.active)
            .map(|(actor, level)| (*actor, level.smoothed))
            .collect();
        actors.sort_by(|(a_actor, a), (b_actor, b)| {
            // Keep the preceding order when two levels are close enough that
            // sample noise could otherwise rearrange tiles each interval.
            if (a - b).abs() < 3.0 {
                let a_rank = self.published.iter().position(|id| id == a_actor);
                let b_rank = self.published.iter().position(|id| id == b_actor);
                if let (Some(a_rank), Some(b_rank)) = (a_rank, b_rank) {
                    return a_rank.cmp(&b_rank);
                }
            }
            b.partial_cmp(a).unwrap_or(Ordering::Equal)
        });
        let next: Vec<_> = actors
            .into_iter()
            .take(MAX_SPEAKERS)
            .map(|(actor, _)| actor)
            .collect();
        if next == self.published {
            return None;
        }
        if !force
            && self
                .last_emitted
                .is_some_and(|last| now.duration_since(last) < MIN_EMIT_INTERVAL)
        {
            return None;
        }
        self.published = next.clone();
        self.last_emitted = Some(now);
        Some(SpeakerSnapshot {
            room_id,
            actor_ids: next,
        })
    }
}
