use super::{
    SpeakerHub,
    tracker::{MIN_EMIT_INTERVAL, RoomSpeakers},
};
use mahfel_domain::{ActorId, RoomId};
use std::time::{Duration, Instant};

#[test]
fn overlapping_producers_are_aggregated_by_actor() {
    let alice = ActorId::new();
    let bob = ActorId::new();
    let mut room = RoomSpeakers::default();
    room.add_producer("a1".to_owned(), alice);
    room.add_producer("a2".to_owned(), alice);
    room.add_producer("b".to_owned(), bob);
    let samples = [
        ("a1".to_owned(), -50),
        ("a2".to_owned(), -59),
        ("b".to_owned(), -55),
    ];
    room.observe(&samples);
    room.observe(&samples);
    let snapshot = room
        .changed_snapshot(RoomId::new(), Instant::now(), false)
        .expect("two speakers become active");
    assert_eq!(snapshot.actor_ids.len(), 2);
    assert_eq!(snapshot.actor_ids[0], alice);
    assert!(snapshot.actor_ids.contains(&bob));
}

#[test]
fn a_short_gap_does_not_remove_a_speaker() {
    let actor = ActorId::new();
    let mut room = RoomSpeakers::default();
    room.add_producer("mic".to_owned(), actor);
    let loud = [("mic".to_owned(), -50)];
    room.observe(&loud);
    room.observe(&loud);
    let room_id = RoomId::new();
    let start = Instant::now();
    assert_eq!(
        room.changed_snapshot(room_id, start, false)
            .expect("entered")
            .actor_ids,
        vec![actor]
    );
    room.observe(&[]);
    room.observe(&[]);
    assert!(
        room.changed_snapshot(room_id, start + MIN_EMIT_INTERVAL, false)
            .is_none()
    );
    room.observe(&[]);
    assert!(
        room.changed_snapshot(room_id, start + MIN_EMIT_INTERVAL, false)
            .expect("left")
            .actor_ids
            .is_empty()
    );
}

#[test]
fn sample_changes_are_rate_limited() {
    let alice = ActorId::new();
    let bob = ActorId::new();
    let mut room = RoomSpeakers::default();
    room.add_producer("alice".to_owned(), alice);
    room.add_producer("bob".to_owned(), bob);
    let start = Instant::now();
    room.observe(&[("alice".to_owned(), -50)]);
    room.observe(&[("alice".to_owned(), -50)]);
    assert!(room.changed_snapshot(RoomId::new(), start, false).is_some());
    room.observe(&[("alice".to_owned(), -50), ("bob".to_owned(), -48)]);
    room.observe(&[("alice".to_owned(), -50), ("bob".to_owned(), -48)]);
    assert!(
        room.changed_snapshot(RoomId::new(), start + Duration::from_millis(100), false)
            .is_none()
    );
    assert_eq!(
        room.changed_snapshot(RoomId::new(), start + MIN_EMIT_INTERVAL, false)
            .expect("delayed update")
            .actor_ids
            .len(),
        2
    );
}

#[test]
fn silence_and_producer_close_clear_immediately() {
    let hub = SpeakerHub::new();
    let room_id = RoomId::new();
    let actor = ActorId::new();
    hub.add_room(room_id);
    hub.add_producer(room_id, "mic".to_owned(), actor);
    let (mut receiver, _) = hub.subscribe_with_snapshots();
    let loud = [("mic".to_owned(), -50)];
    hub.observe(room_id, &loud);
    hub.observe(room_id, &loud);
    assert_eq!(
        receiver.try_recv().expect("active snapshot").actor_ids,
        vec![actor]
    );
    hub.silence(room_id);
    assert!(
        receiver
            .try_recv()
            .expect("silence snapshot")
            .actor_ids
            .is_empty()
    );
    hub.observe(room_id, &loud);
    hub.observe(room_id, &loud);
    hub.remove_producer(room_id, "mic");
    assert!(hub.subscribe_with_snapshots().1[0].actor_ids.is_empty());
    let (mut receiver, _) = hub.subscribe_with_snapshots();
    hub.clear_room(room_id);
    assert!(
        receiver
            .try_recv()
            .expect("room reset")
            .actor_ids
            .is_empty()
    );
    assert_eq!(hub.subscribe_with_snapshots().1.len(), 1);
}
