use super::*;
use crate::http::ws::envelope::ChatEventPayload;
use mahfel_domain::ParticipantId;

fn registry() -> ConnectionRegistry {
    let metrics = WsMetrics::register(&prometheus::Registry::new())
        .unwrap_or_else(|_| panic!("register ws metrics"));
    ConnectionRegistry::new(16, metrics)
}

/// Register a connection and put it in a session as an actor.
fn connected(
    registry: &ConnectionRegistry,
    session: SessionId,
    actor: ActorId,
) -> (Uuid, RegisteredConnection) {
    let id = Uuid::new_v4();
    let (sender, _receiver) = mpsc::channel(4);
    let guard = registry.register(id, actor, sender);
    registry.joined(id, RoomId::new(), session, ParticipantId::new());
    (id, guard)
}

fn detach(registry: &ConnectionRegistry, id: Uuid) {
    let handle = registry.participation(id).expect("joined connection");
    assert!(registry.left_if_current(id, handle));
}

#[tokio::test]
async fn two_replica_hubs_apply_one_event_once_and_bound_slow_clients() {
    let first = registry();
    let second = registry();
    let room = RoomId::new();
    let actor = ActorId::new();
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let (sender_a, mut receiver_a) = mpsc::channel(1);
    let (sender_b, mut receiver_b) = mpsc::channel(1);
    let _guard_a = first.register(id_a, actor, sender_a);
    let _guard_b = second.register(id_b, actor, sender_b);
    assert!(first.subscribe_chat(id_a, room, 0));
    assert!(second.subscribe_chat(id_b, room, 0));
    let event = ServerMessage::ChatEvent {
        event: Box::new(ChatEventPayload {
            room_id: room.to_string(),
            sequence: 1,
            event_id: Uuid::new_v4().to_string(),
            kind: "created".to_owned(),
            message_id: Uuid::new_v4().to_string(),
            parent_id: None,
            author_id: actor.to_string(),
            reaction_actor_id: None,
            reaction_emoji: None,
            content: Some("hello".to_owned()),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            edited_at: None,
            deleted_at: None,
        }),
    };
    first.send_chat_event(id_a, room, 1, event.clone());
    second.send_chat_event(id_b, room, 1, event.clone());
    first.send_chat_event(id_a, room, 1, event.clone());
    second.send_chat_event(id_b, room, 1, event.clone());
    assert!(matches!(
        receiver_a.try_recv(),
        Ok(Outbound::Message(ServerMessage::ChatEvent { event })) if event.sequence == 1
    ));
    assert!(matches!(
        receiver_b.try_recv(),
        Ok(Outbound::Message(ServerMessage::ChatEvent { event })) if event.sequence == 1
    ));
    assert!(receiver_a.try_recv().is_err());
    assert!(receiver_b.try_recv().is_err());

    first.send_chat_event(id_a, room, 2, event.clone());
    first.send_chat_event(id_a, room, 3, event);
    assert!(
        first.chat_subscriptions().is_empty(),
        "a full outbound queue disconnects the stalled client"
    );
    assert!(
        !second.chat_subscriptions().is_empty(),
        "the other replica remains live"
    );
}

#[test]
fn a_second_device_does_not_add_a_participant() {
    // Both connections map to the same actor, which is what makes a second
    // device share one participant rather than creating another.
    let registry = registry();
    let session = SessionId::new();
    let actor = ActorId::new();
    let (_first, _guard_a) = connected(&registry, session, actor);
    let (_second, _guard_b) = connected(&registry, session, actor);

    assert_eq!(
        registry.connection_count_for_participant(session, actor),
        2,
        "one participant, two connections"
    );
}

#[test]
fn a_participant_stays_present_while_one_connection_is_live() {
    let registry = registry();
    let session = SessionId::new();
    let actor = ActorId::new();
    let (first, guard_a) = connected(&registry, session, actor);
    let (_second, _guard_b) = connected(&registry, session, actor);

    detach(&registry, first);
    drop(guard_a);
    assert_eq!(
        registry.connection_count_for_participant(session, actor),
        1,
        "presence holds while another connection is live"
    );
}

#[test]
fn a_participant_is_absent_once_the_last_connection_goes() {
    let registry = registry();
    let session = SessionId::new();
    let actor = ActorId::new();
    let (first, guard_a) = connected(&registry, session, actor);
    let (second, guard_b) = connected(&registry, session, actor);

    detach(&registry, first);
    detach(&registry, second);
    drop(guard_a);
    drop(guard_b);
    assert_eq!(
        registry.connection_count_for_participant(session, actor),
        0,
        "the participant is absent only when every connection is gone"
    );
}

#[test]
fn another_participants_connections_are_not_counted() {
    let registry = registry();
    let session = SessionId::new();
    let actor = ActorId::new();
    let other = ActorId::new();
    let (_mine, _guard_a) = connected(&registry, session, actor);
    let (_theirs, _guard_b) = connected(&registry, session, other);

    assert_eq!(registry.connection_count_for_participant(session, actor), 1);
    assert_eq!(registry.connection_count_for_participant(session, other), 1);
}

#[test]
fn the_same_actor_in_another_session_is_not_counted() {
    let registry = registry();
    let session = SessionId::new();
    let elsewhere = SessionId::new();
    let actor = ActorId::new();
    let (_here, _guard_a) = connected(&registry, session, actor);
    let (_there, _guard_b) = connected(&registry, elsewhere, actor);

    assert_eq!(
        registry.connection_count_for_participant(session, actor),
        1,
        "presence is per session, not per actor"
    );
}

#[test]
fn a_connection_that_has_not_joined_is_not_present() {
    let registry = registry();
    let session = SessionId::new();
    let actor = ActorId::new();
    let id = Uuid::new_v4();
    let (sender, _receiver) = mpsc::channel(4);
    let _guard = registry.register(id, actor, sender);

    assert_eq!(
        registry.connection_count_for_participant(session, actor),
        0,
        "an authenticated connection is not a participant until it joins"
    );
}

#[test]
fn a_stale_generation_cannot_detach_a_rejoined_connection() {
    let registry = registry();
    let id = Uuid::new_v4();
    let (sender, _receiver) = mpsc::channel(4);
    let _registration = registry.register(id, ActorId::new(), sender);
    let session_id = SessionId::new();
    let room_id = RoomId::new();
    let participant_id = ParticipantId::new();
    registry.joined(id, room_id, session_id, participant_id);
    let stale = registry.participation(id).expect("first generation");
    registry.joined(id, room_id, session_id, participant_id);

    assert!(!registry.left_if_current(id, stale));
    assert_eq!(
        registry
            .participation(id)
            .map(|handle| handle.participant_id),
        Some(participant_id)
    );
}
