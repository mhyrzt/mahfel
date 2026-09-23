use std::{sync::Arc, time::Duration};

use mahfel_application::ports::{MediaNodeRecord, MediaNodeRegistry, MediaPlacementLease};
use mahfel_domain::{NodeId, SessionId};
use mahfel_redis::{RedisConnectionSettings, RedisCoordination, RedisMetrics};
use prometheus::Registry;
use testcontainers::{
    ContainerAsync, GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use uuid::Uuid;

async fn redis_target()
-> Result<Option<(String, Option<ContainerAsync<GenericImage>>)>, Box<dyn std::error::Error>> {
    if let Ok(url) = std::env::var("MAHFEL_TEST_REDIS_URL") {
        return Ok(Some((url, None)));
    }
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!(
            "MAHFEL_TEST_REDIS_URL and MAHFEL_TESTCONTAINERS unset; skipping Redis placement test"
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
async fn concurrent_claims_keep_one_node_and_one_reserved_slot()
-> Result<(), Box<dyn std::error::Error>> {
    let Some((url, _container)) = redis_target().await? else {
        return Ok(());
    };
    let settings = RedisConnectionSettings {
        url,
        key_prefix: format!("mahfel-placement-test-{}", Uuid::new_v4().simple()),
        connect_timeout: Duration::from_secs(2),
        command_timeout: Duration::from_secs(2),
    };
    let redis = Arc::new(
        RedisCoordination::connect(&settings, RedisMetrics::register(&Registry::new())?).await?,
    );
    let node_a = NodeId::from_deployment("node-a").expect("valid node id");
    let node_b = NodeId::from_deployment("node-b").expect("valid node id");
    for id in [&node_a, &node_b] {
        redis
            .heartbeat(
                &MediaNodeRecord {
                    node_id: id.clone(),
                    grpc_url: format!("http://{id}:50051"),
                    max_rooms: 1,
                    active_workers: 1,
                    active_rooms: 0,
                    transports: 0,
                    draining: false,
                },
                Duration::from_secs(20),
            )
            .await?;
    }
    let session = SessionId::new();
    let candidates = Arc::new(vec![node_a.clone(), node_b.clone()]);
    let mut tasks = Vec::new();
    for index in 0..32 {
        let redis = Arc::clone(&redis);
        let candidates = Arc::clone(&candidates);
        tasks.push(tokio::spawn(async move {
            let order = if index % 2 == 0 {
                candidates.as_slice().to_vec()
            } else {
                candidates.iter().rev().cloned().collect()
            };
            redis
                .acquire(session, &order, Duration::from_secs(60))
                .await
        }));
    }
    let mut winners = Vec::new();
    for task in tasks {
        winners.push(task.await??.expect("both nodes have capacity"));
    }
    assert!(winners.iter().all(|winner| winner == &winners[0]));
    assert_eq!(
        redis.live_node(&winners[0]).await?.map(|node| node.node_id),
        Some(winners[0].clone()),
    );

    // A second session consumes the other slot; a third cannot overbook either
    // node even while the media heartbeat still reports zero active rooms.
    let second = redis
        .acquire(
            SessionId::new(),
            candidates.as_slice(),
            Duration::from_secs(60),
        )
        .await?
        .expect("one free node remains");
    assert_ne!(second, winners[0]);
    assert_eq!(
        redis
            .acquire(
                SessionId::new(),
                candidates.as_slice(),
                Duration::from_secs(60)
            )
            .await?,
        None,
    );
    let draining = NodeId::from_deployment("draining").expect("valid node id");
    redis
        .heartbeat(
            &MediaNodeRecord {
                node_id: draining.clone(),
                grpc_url: "http://draining:50051".to_owned(),
                max_rooms: 10,
                active_workers: 1,
                active_rooms: 0,
                transports: 0,
                draining: true,
            },
            Duration::from_secs(20),
        )
        .await?;
    assert!(
        redis
            .live_node(&draining)
            .await?
            .expect("live node")
            .draining
    );
    assert_eq!(
        redis
            .acquire(
                SessionId::new(),
                std::slice::from_ref(&draining),
                Duration::from_secs(60)
            )
            .await?,
        None,
        "draining node cannot receive a new session",
    );

    let expired = NodeId::from_deployment("expired").expect("valid node id");
    redis
        .heartbeat(
            &MediaNodeRecord {
                node_id: expired.clone(),
                grpc_url: "http://expired:50051".to_owned(),
                max_rooms: 10,
                active_workers: 1,
                active_rooms: 0,
                transports: 0,
                draining: false,
            },
            Duration::from_millis(100),
        )
        .await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(redis.live_node(&expired).await?.is_none());
    assert_eq!(
        redis
            .acquire(SessionId::new(), &[expired], Duration::from_secs(60))
            .await?,
        None
    );
    Ok(())
}
