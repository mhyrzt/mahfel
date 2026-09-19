use mahfel_application::ports::CoordinationStore;
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
    coordination.publish(&channel, b"hello").await?;
    let message = tokio::time::timeout(Duration::from_secs(3), receiver.recv())
        .await
        .expect("message within timeout")
        .expect("message present");
    assert_eq!(message.channel, channel);
    assert_eq!(message.payload, b"hello".to_vec());

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
