use super::error::RtcError;
use super::room::RoomMap;
use super::transport::TransportMap;
use mahfel_domain::{ActorId, RoomId};
use mediasoup::prelude::{MediaKind, Producer, ProducerOptions, RtpParameters, Transport};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct ProducerEntry {
    pub(super) producer: Producer,
    pub(super) room_id: RoomId,
    pub(super) transport_id: String,
    pub(super) participant_id: ActorId,
}

pub(super) type ProducerMap = Arc<RwLock<HashMap<String, ProducerEntry>>>;

#[allow(clippy::too_many_arguments)]
pub(super) async fn create_producer(
    rooms: &RoomMap,
    transports: &TransportMap,
    producers: &ProducerMap,
    room_id: RoomId,
    transport_id: &str,
    participant_id: ActorId,
    kind: MediaKind,
    rtp_parameters_json: &str,
) -> Result<String, RtcError> {
    let rtp_parameters: RtpParameters = serde_json::from_str(rtp_parameters_json)
        .map_err(|e| RtcError::RtpParameters(e.to_string()))?;

    {
        let map = rooms.read().await;
        if !map.contains_key(&room_id) {
            return Err(RtcError::RoomNotFound { room_id });
        }
    }

    let transport = {
        let map = transports.read().await;
        let entry = map
            .get(transport_id)
            .ok_or_else(|| RtcError::TransportNotFound {
                transport_id: transport_id.to_owned(),
            })?;
        if entry.room_id != room_id || entry.participant_id != participant_id {
            return Err(RtcError::TransportNotFound {
                transport_id: transport_id.to_owned(),
            });
        }
        entry.transport.clone()
    };

    let producer = transport
        .produce(ProducerOptions::new(kind, rtp_parameters))
        .await
        .map_err(|e| RtcError::Producer(e.to_string()))?;

    let producer_id = producer.id().to_string();
    producers.write().await.insert(
        producer_id.clone(),
        ProducerEntry {
            producer,
            room_id,
            transport_id: transport_id.to_owned(),
            participant_id,
        },
    );
    Ok(producer_id)
}

/// Pause one producer without destroying it.
///
/// Pausing rather than closing is what lets an enforced mute map onto media
/// state one-to-one: resuming needs no renegotiation, and the caller's durable
/// mute record never points at a producer that no longer exists.
pub(super) async fn pause_producer(
    producers: &ProducerMap,
    room_id: RoomId,
    producer_id: &str,
) -> Result<(), RtcError> {
    let map = producers.read().await;
    let entry = map
        .get(producer_id)
        .ok_or_else(|| RtcError::ProducerNotFound {
            producer_id: producer_id.to_owned(),
        })?;
    // A producer that belongs to another room is not this caller's to touch, so
    // it is reported as absent rather than as a permission error.
    if entry.room_id != room_id {
        return Err(RtcError::ProducerNotFound {
            producer_id: producer_id.to_owned(),
        });
    }
    entry
        .producer
        .pause()
        .await
        .map_err(|e| RtcError::Producer(e.to_string()))?;
    Ok(())
}

/// Resume a producer that was paused.
pub(super) async fn resume_producer(
    producers: &ProducerMap,
    room_id: RoomId,
    producer_id: &str,
) -> Result<(), RtcError> {
    let map = producers.read().await;
    let entry = map
        .get(producer_id)
        .ok_or_else(|| RtcError::ProducerNotFound {
            producer_id: producer_id.to_owned(),
        })?;
    if entry.room_id != room_id {
        return Err(RtcError::ProducerNotFound {
            producer_id: producer_id.to_owned(),
        });
    }
    entry
        .producer
        .resume()
        .await
        .map_err(|e| RtcError::Producer(e.to_string()))?;
    Ok(())
}

/// Close every producer owned by one participant in one room.
///
/// Removal is terminal, so closing is correct here rather than pausing: the
/// participant will not be resumed. Returns how many were closed, which is zero
/// for a participant that published nothing.
///
/// A producer is closed by dropping it — `Producer` is `#[must_use]` precisely
/// because it closes on drop, and mediasoup keeps `close` private for that
/// reason. Removing the entry is therefore the close, and it also unregisters
/// the producer in the same step, so a concurrent close cannot find it and a
/// partially closed producer cannot be left behind.
pub(super) async fn close_participant(
    producers: &ProducerMap,
    room_id: RoomId,
    participant_id: ActorId,
) -> Result<usize, RtcError> {
    let mut map = producers.write().await;
    let owned = owned_producer_ids(&map, room_id, participant_id);

    let mut closed = 0;
    for producer_id in owned {
        if map.remove(&producer_id).is_some() {
            closed += 1;
        }
    }
    Ok(closed)
}

/// Which of a participant's producers a close should remove.
fn owned_producer_ids(
    producers: &HashMap<String, ProducerEntry>,
    room_id: RoomId,
    participant_id: ActorId,
) -> Vec<String> {
    producers
        .iter()
        .filter(|(_, entry)| owned_by(entry.room_id, entry.participant_id, room_id, participant_id))
        .map(|(id, _)| id.clone())
        .collect()
}

/// Whether one entry is a producer this close should remove.
///
/// The rule the whole operation rests on: a producer is removed only for the
/// named participant, and only in the named room. A producer in another room is
/// not this caller's to close even for the right participant.
///
/// It takes the entry's two fields rather than the entry itself, which is what
/// makes the rule testable: a `ProducerEntry` cannot be constructed outside
/// mediasoup, since `Producer::new` is private and needs a live worker. Four
/// `RoomId`/`ActorId` arguments are easy to transpose, so the first two are
/// always the entry's own fields and the last two are the request's.
fn owned_by(
    entry_room_id: RoomId,
    entry_participant_id: ActorId,
    room_id: RoomId,
    participant_id: ActorId,
) -> bool {
    entry_room_id == room_id && entry_participant_id == participant_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_producer_is_owned_only_in_its_own_room() {
        let room = RoomId::new();
        let other_room = RoomId::new();
        let alice = ActorId::new();

        assert!(
            owned_by(room, alice, room, alice),
            "the participant's producer in this room is selected"
        );
        assert!(
            !owned_by(room, alice, other_room, alice),
            "a producer in another room must not be closed, even for the same participant"
        );
    }

    #[test]
    fn a_producer_is_owned_only_by_its_own_participant() {
        let room = RoomId::new();
        let alice = ActorId::new();
        let bob = ActorId::new();

        assert!(owned_by(room, alice, room, alice));
        assert!(
            !owned_by(room, alice, room, bob),
            "another participant's producer must not be closed"
        );
    }

    #[test]
    fn a_producer_in_another_room_belonging_to_another_participant_is_not_owned() {
        let room = RoomId::new();
        let other_room = RoomId::new();
        let alice = ActorId::new();
        let bob = ActorId::new();

        assert!(
            !owned_by(other_room, bob, room, alice),
            "both fields must match; one matching is not enough"
        );
    }

    #[test]
    fn a_room_that_matches_with_a_participant_that_does_not_is_not_owned() {
        // Guards the argument order: if the entry's room were compared against
        // the requested participant, this case would report ownership.
        let room = RoomId::new();
        let alice = ActorId::new();
        assert!(
            !owned_by(room, alice, room, ActorId::new()),
            "a matching room alone must not select a producer"
        );
    }
}
