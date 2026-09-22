use super::error::RtcError;
use super::room::RoomMap;
use super::speaker::SpeakerHub;
use super::transport::TransportMap;
use mahfel_domain::{ActorId, ProducerSource, RoomId};
use mediasoup::{
    prelude::{MediaKind, Producer, ProducerOptions, RtpParameters, Transport},
    rtp_observer::{RtpObserver, RtpObserverAddProducerOptions},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct ProducerEntry {
    pub(super) producer: Producer,
    pub(super) room_id: RoomId,
    pub(super) transport_id: String,
    pub(super) participant_id: ActorId,
    pub(super) kind: MediaKind,
    pub(super) source: ProducerSource,
}

pub(super) type ProducerMap = Arc<RwLock<HashMap<String, ProducerEntry>>>;

#[allow(clippy::too_many_arguments)]
pub(super) async fn create_producer(
    rooms: &RoomMap,
    transports: &TransportMap,
    producers: &ProducerMap,
    speakers: &SpeakerHub,
    room_id: RoomId,
    transport_id: &str,
    participant_id: ActorId,
    kind: MediaKind,
    source: ProducerSource,
    rtp_parameters_json: &str,
) -> Result<String, RtcError> {
    let rtp_parameters: RtpParameters = serde_json::from_str(rtp_parameters_json)
        .map_err(|e| RtcError::RtpParameters(e.to_string()))?;

    let observer = {
        let map = rooms.read().await;
        map.get(&room_id)
            .ok_or(RtcError::RoomNotFound { room_id })?
            .audio_observer
            .clone()
    };

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
    if kind == MediaKind::Audio && source == ProducerSource::Microphone {
        speakers.add_producer(room_id, producer_id.clone(), participant_id);
        if let Err(error) = observer
            .add_producer(RtpObserverAddProducerOptions::new(producer.id()))
            .await
        {
            speakers.remove_producer(room_id, &producer_id);
            return Err(RtcError::Observer(error.to_string()));
        }
    }
    producers.write().await.insert(
        producer_id.clone(),
        ProducerEntry {
            producer,
            room_id,
            transport_id: transport_id.to_owned(),
            participant_id,
            kind,
            source,
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
    speakers: &SpeakerHub,
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
    speakers.remove_producer(room_id, producer_id);
    Ok(())
}

/// Resume a producer that was paused.
pub(super) async fn resume_producer(
    producers: &ProducerMap,
    speakers: &SpeakerHub,
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
    if entry.kind == MediaKind::Audio && entry.source == ProducerSource::Microphone {
        speakers.add_producer(room_id, producer_id.to_owned(), entry.participant_id);
    }
    Ok(())
}

/// Pause every producer a participant owns for one source.
///
/// Source targeting is what makes muting a microphone distinct from stopping a
/// camera or a screen share, which are all video or audio to the SFU. Returns
/// how many were paused, which is zero when they published nothing of that
/// source.
pub(super) async fn pause_participant_source(
    producers: &ProducerMap,
    speakers: &SpeakerHub,
    room_id: RoomId,
    participant_id: ActorId,
    source: ProducerSource,
) -> Result<usize, RtcError> {
    let map = producers.read().await;
    let selected = owned_producer_ids(&map, room_id, participant_id, Some(source));

    let mut paused = 0;
    for producer_id in selected {
        let Some(entry) = map.get(&producer_id) else {
            continue;
        };
        entry
            .producer
            .pause()
            .await
            .map_err(|e| RtcError::Producer(e.to_string()))?;
        speakers.remove_producer(room_id, &producer_id);
        paused += 1;
    }
    Ok(paused)
}

/// Resume every producer a participant owns for one source.
pub(super) async fn resume_participant_source(
    producers: &ProducerMap,
    speakers: &SpeakerHub,
    room_id: RoomId,
    participant_id: ActorId,
    source: ProducerSource,
) -> Result<usize, RtcError> {
    let map = producers.read().await;
    let selected = owned_producer_ids(&map, room_id, participant_id, Some(source));

    let mut resumed = 0;
    for producer_id in selected {
        let Some(entry) = map.get(&producer_id) else {
            continue;
        };
        entry
            .producer
            .resume()
            .await
            .map_err(|e| RtcError::Producer(e.to_string()))?;
        if entry.kind == MediaKind::Audio && entry.source == ProducerSource::Microphone {
            speakers.add_producer(room_id, producer_id.clone(), entry.participant_id);
        }
        resumed += 1;
    }
    Ok(resumed)
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
    speakers: &SpeakerHub,
    room_id: RoomId,
    participant_id: ActorId,
) -> Result<usize, RtcError> {
    let mut map = producers.write().await;
    // Removal ends participation, so every source goes; source targeting is for
    // mute, which is a narrower action.
    let owned = owned_producer_ids(&map, room_id, participant_id, None);

    let mut closed = 0;
    for producer_id in owned {
        if map.remove(&producer_id).is_some() {
            speakers.remove_producer(room_id, &producer_id);
            closed += 1;
        }
    }
    Ok(closed)
}

/// Close every producer a participant owns for one source.
///
/// Removal closes every source; this closes one, which is what a capability
/// withdrawal needs when only part of a participant's media is no longer
/// permitted. Returns how many were closed.
pub(super) async fn close_participant_source(
    producers: &ProducerMap,
    speakers: &SpeakerHub,
    room_id: RoomId,
    participant_id: ActorId,
    source: ProducerSource,
) -> Result<usize, RtcError> {
    let mut map = producers.write().await;
    let owned = owned_producer_ids(&map, room_id, participant_id, Some(source));

    let mut closed = 0;
    for producer_id in owned {
        if map.remove(&producer_id).is_some() {
            speakers.remove_producer(room_id, &producer_id);
            closed += 1;
        }
    }
    Ok(closed)
}

/// Which of a participant's producers a selection should take.
fn owned_producer_ids(
    producers: &HashMap<String, ProducerEntry>,
    room_id: RoomId,
    participant_id: ActorId,
    source: Option<ProducerSource>,
) -> Vec<String> {
    producers
        .iter()
        .filter(|(_, entry)| {
            owned_by(
                entry.room_id,
                entry.participant_id,
                entry.source,
                room_id,
                participant_id,
                source,
            )
        })
        .map(|(id, _)| id.clone())
        .collect()
}

/// Whether one entry matches a moderation selection.
///
/// A selection names a source, or all of them when `source` is `None`, so
/// muting audio and stopping a screen share are separate operations rather than
/// both ending the participant's video.
///
/// It takes the entry's fields rather than the entry itself, which is what
/// makes the rule testable: a `ProducerEntry` cannot be constructed outside
/// mediasoup, since `Producer::new` is private and needs a live worker. Several
/// `RoomId`/`ActorId` arguments are easy to transpose, so the entry's own
/// fields always come first and the request's after them.
fn owned_by(
    entry_room_id: RoomId,
    entry_participant_id: ActorId,
    entry_source: ProducerSource,
    room_id: RoomId,
    participant_id: ActorId,
    source: Option<ProducerSource>,
) -> bool {
    entry_room_id == room_id
        && entry_participant_id == participant_id
        && source.is_none_or(|wanted| entry_source == wanted)
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
            owned_by(room, alice, ProducerSource::Microphone, room, alice, None),
            "the participant's producer in this room is selected"
        );
        assert!(
            !owned_by(
                room,
                alice,
                ProducerSource::Microphone,
                other_room,
                alice,
                None
            ),
            "a producer in another room must not be selected, even for the same participant"
        );
    }

    #[test]
    fn a_producer_is_owned_only_by_its_own_participant() {
        let room = RoomId::new();
        let alice = ActorId::new();
        let bob = ActorId::new();

        assert!(owned_by(
            room,
            alice,
            ProducerSource::Camera,
            room,
            alice,
            None
        ));
        assert!(
            !owned_by(room, alice, ProducerSource::Camera, room, bob, None),
            "another participant's producer must not be selected"
        );
    }

    #[test]
    fn every_source_is_selected_when_none_is_named() {
        let room = RoomId::new();
        let alice = ActorId::new();
        for source in [
            ProducerSource::Microphone,
            ProducerSource::Camera,
            ProducerSource::Screen,
        ] {
            assert!(
                owned_by(room, alice, source, room, alice, None),
                "an unnamed source selects all of them, which is what removal needs"
            );
        }
    }

    #[test]
    fn only_the_named_source_is_selected_when_one_is_named() {
        let room = RoomId::new();
        let alice = ActorId::new();

        assert!(owned_by(
            room,
            alice,
            ProducerSource::Microphone,
            room,
            alice,
            Some(ProducerSource::Microphone)
        ));
        assert!(
            !owned_by(
                room,
                alice,
                ProducerSource::Camera,
                room,
                alice,
                Some(ProducerSource::Microphone)
            ),
            "muting the microphone must not touch the camera"
        );
    }

    #[test]
    fn stopping_a_screen_share_leaves_the_camera_alone() {
        // The case that makes the source field necessary: both are video to the
        // SFU, so a kind-only selection could not tell them apart.
        let room = RoomId::new();
        let alice = ActorId::new();

        assert!(owned_by(
            room,
            alice,
            ProducerSource::Screen,
            room,
            alice,
            Some(ProducerSource::Screen)
        ));
        assert!(
            !owned_by(
                room,
                alice,
                ProducerSource::Camera,
                room,
                alice,
                Some(ProducerSource::Screen)
            ),
            "stopping a screen share must not stop the camera"
        );
    }

    #[test]
    fn a_room_that_matches_with_a_participant_that_does_not_is_not_owned() {
        // Guards the argument order: if the entry's room were compared against
        // the requested participant, this case would report ownership.
        let room = RoomId::new();
        let alice = ActorId::new();
        assert!(
            !owned_by(
                room,
                alice,
                ProducerSource::Camera,
                room,
                ActorId::new(),
                None
            ),
            "a matching room alone must not select a producer"
        );
    }
}
