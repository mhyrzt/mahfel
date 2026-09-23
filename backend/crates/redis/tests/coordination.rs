use mahfel_application::ports::{ChatActivityStore, CoordinationStore};
use mahfel_domain::{ActorId, ParticipantId, RoomId, SessionId};
use mahfel_redis::{RedisConnectionSettings, RedisCoordination, RedisError, RedisMetrics};
use prometheus::Registry;
use std::time::Duration;
use testcontainers::{
    ContainerAsync, GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use uuid::Uuid;

type Container = ContainerAsync<GenericImage>;

fn settings(url: &str, connect_timeout: Duration) -> RedisConnectionSettings {
    RedisConnectionSettings {
        url: url.to_owned(),
        key_prefix: format!("mahfel-test-{}", Uuid::new_v4().simple()),
        connect_timeout,
        command_timeout: Duration::from_secs(2),
    }
}

fn metrics() -> RedisMetrics {
    RedisMetrics::register(&Registry::new()).expect("register metrics")
}

/// Resolve a Redis target: an explicit URL, a disposable container, or skip.
async fn redis_target() -> Result<Option<(String, Option<Container>)>, Box<dyn std::error::Error>> {
    if let Ok(url) = std::env::var("MAHFEL_TEST_REDIS_URL") {
        return Ok(Some((url, None)));
    }
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!(
            "MAHFEL_TEST_REDIS_URL and MAHFEL_TESTCONTAINERS unset; skipping Redis integration test"
        );
        return Ok(None);
    }
    let container = GenericImage::new("redis", "7.4-alpine")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await?;
    let port = container.get_host_port_ipv4(6379).await?;
    Ok(Some((format!("redis://127.0.0.1:{port}"), Some(container))))
}

#[tokio::test]
async fn values_expire_and_pubsub_round_trips() -> Result<(), Box<dyn std::error::Error>> {
    let Some((url, _container)) = redis_target().await? else {
        return Ok(());
    };
    let coordination =
        RedisCoordination::connect(&settings(&url, Duration::from_secs(2)), metrics()).await?;

    // A value round-trips and is removable.
    let key = coordination.keys().key("test", &Uuid::new_v4().to_string());
    coordination
        .set_with_ttl(&key, b"value", Duration::from_secs(30))
        .await?;
    assert_eq!(coordination.get(&key).await?, Some(b"value".to_vec()));
    coordination.delete(&key).await?;
    assert_eq!(coordination.get(&key).await?, None);

    // An ephemeral key expires on its own.
    let expiring = coordination.keys().key("test", &Uuid::new_v4().to_string());
    coordination
        .set_with_ttl(&expiring, b"soon-gone", Duration::from_millis(50))
        .await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        coordination.get(&expiring).await?,
        None,
        "the key must expire without an explicit delete"
    );

    // Pub/sub delivers the payload on a bounded queue.
    let channel = coordination.keys().channel("test-events");
    let mut receiver = coordination.subscribe(&channel).await?;
    let mut second_replica = coordination.subscribe(&channel).await?;
    coordination.publish(&channel, b"hello").await?;
    let message = tokio::time::timeout(Duration::from_secs(3), receiver.recv())
        .await
        .expect("message within timeout")
        .expect("message present");
    assert_eq!(message.channel, channel);
    assert_eq!(message.payload, b"hello".to_vec());
    let duplicate = tokio::time::timeout(Duration::from_secs(3), second_replica.recv())
        .await
        .expect("second replica receives publish")
        .expect("second message present");
    assert_eq!(duplicate.payload, b"hello".to_vec());

    let logical_counter = format!("rate-test:{}", Uuid::new_v4());
    assert_eq!(
        coordination
            .increment_with_ttl(&logical_counter, Duration::from_millis(100))
            .await?,
        1
    );
    assert_eq!(
        coordination
            .increment_with_ttl(&logical_counter, Duration::from_millis(100))
            .await?,
        2
    );
    let physical_counter = coordination.keys().key("counter", &logical_counter);
    assert!(coordination.get(&physical_counter).await?.is_some());
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert_eq!(
        coordination
            .increment_with_ttl(&logical_counter, Duration::from_millis(100))
            .await?,
        1,
        "a fixed window resets after its first write expires"
    );

    Ok(())
}

#[tokio::test]
async fn outage_maps_to_unavailable() {
    // Nothing listens on port 1, so the connection is refused.
    let error = match RedisCoordination::connect(
        &settings("redis://127.0.0.1:1", Duration::from_millis(500)),
        metrics(),
    )
    .await
    {
        Ok(_) => panic!("connection must fail"),
        Err(error) => error,
    };
    assert!(
        matches!(error, RedisError::Unavailable(_)),
        "an unreachable backend must be Unavailable, got {error:?}"
    );
}

#[tokio::test]
async fn chat_activity_aggregates_and_expires() -> Result<(), Box<dyn std::error::Error>> {
    let Some((url, _container)) = redis_target().await? else {
        return Ok(());
    };
    let coordination =
        RedisCoordination::connect(&settings(&url, Duration::from_secs(2)), metrics()).await?;
    let room = RoomId::new();
    let first = ActorId::new();
    let second = ActorId::new();
    coordination
        .heartbeat(room, first, Duration::from_millis(150))
        .await?;
    coordination
        .heartbeat(room, first, Duration::from_millis(150))
        .await?;
    coordination
        .heartbeat(room, second, Duration::from_millis(150))
        .await?;
    coordination
        .set_typing(room, first, true, Duration::from_millis(100))
        .await?;
    let snapshot = coordination.snapshot(room).await?;
    assert_eq!(
        snapshot.online_count, 2,
        "two devices of one actor count once"
    );
    assert_eq!(snapshot.typing_count, 1);
    tokio::time::sleep(Duration::from_millis(220)).await;
    let expired = coordination.snapshot(room).await?;
    assert_eq!(expired.online_count, 0);
    assert_eq!(expired.typing_count, 0);
    Ok(())
}

#[tokio::test]
async fn session_presence_is_shared_deduplicated_and_resyncs_after_expiry()
-> Result<(), Box<dyn std::error::Error>> {
    let Some((url, _container)) = redis_target().await? else {
        return Ok(());
    };
    let config = settings(&url, Duration::from_secs(2));
    let first = RedisCoordination::connect(&config, metrics()).await?;
    let second = RedisCoordination::connect(&config, metrics()).await?;
    let session = SessionId::new();
    let participant = ParticipantId::new();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let ttl = Duration::from_millis(250);
    assert_eq!(
        first
            .session_presence_join(session, a, participant, ttl)
            .await?,
        1
    );
    assert_eq!(
        second
            .session_presence_join(session, b, participant, ttl)
            .await?,
        2
    );
    assert_eq!(
        first
            .session_presence_snapshot(session)
            .await?
            .participant_ids,
        vec![participant]
    );
    assert_eq!(
        first
            .session_presence_leave(session, a, participant, ttl)
            .await?,
        1
    );
    assert_eq!(
        first
            .session_presence_leave(session, a, participant, ttl)
            .await?,
        1,
        "stale cleanup cannot remove a newer connection on another replica"
    );
    assert_eq!(
        second
            .session_presence_snapshot(session)
            .await?
            .participant_ids,
        vec![participant]
    );
    assert!(
        !first
            .session_presence_renew(session, a, participant, ttl)
            .await?
    );
    assert!(
        second
            .session_presence_renew(session, b, participant, ttl)
            .await?
    );
    tokio::time::sleep(Duration::from_millis(320)).await;
    let expired = first.session_presence_snapshot(session).await?;
    assert!(expired.participant_ids.is_empty());
    assert!(expired.version > 0);
    assert_eq!(
        second
            .session_presence_leave(session, b, participant, ttl)
            .await?,
        0
    );
    Ok(())
}
