use futures_util::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
};
use mahfel_application::jobs::JobEnvelope;
use mahfel_messaging::{
    ConfirmedPublisher, DEAD_QUEUE, JobConsumer, MessagingError, connect, declare_topology,
};
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::test]
async fn confirmed_delivery_and_dead_lettering() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping RabbitMQ integration test");
        return Ok(());
    }
    let broker = GenericImage::new("rabbitmq", "4.2-management-alpine")
        .with_exposed_port(5672.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Server startup complete"))
        .with_env_var("RABBITMQ_DEFAULT_USER", "mahfel")
        .with_env_var("RABBITMQ_DEFAULT_PASS", "mahfel-test")
        .start()
        .await?;
    let port = broker.get_host_port_ipv4(5672).await?;
    let url = format!("amqp://mahfel:mahfel-test@127.0.0.1:{port}/%2f");
    let connection = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match connect(&url).await {
                Ok(connection) => break connection,
                Err(_) => tokio::time::sleep(Duration::from_millis(200)).await,
            }
        }
    })
    .await?;
    declare_topology(&connection).await?;
    declare_topology(&connection).await?;
    let publisher = ConfirmedPublisher::new(&connection).await?;
    let mut consumer = JobConsumer::new(&connection, 1).await?;
    let job = JobEnvelope::file_cleanup(Uuid::new_v4(), OffsetDateTime::now_utc());
    publisher.publish(&job).await?;
    let delivery = tokio::time::timeout(Duration::from_secs(10), consumer.next())
        .await?
        .expect("published job delivered")?;
    let actual: JobEnvelope = serde_json::from_slice(delivery.body())?;
    assert_eq!(actual, job);
    delivery.ack().await?;

    let mut unroutable = job.clone();
    unroutable.job_type = "unknown.kind".to_owned();
    assert!(matches!(
        publisher.publish(&unroutable).await,
        Err(MessagingError::Unroutable)
    ));

    publisher.publish(&job).await?;
    let delivery = tokio::time::timeout(Duration::from_secs(10), consumer.next())
        .await?
        .expect("poison fixture delivered")?;
    delivery.dead_letter().await?;
    let channel = connection.create_channel().await?;
    let mut dead = channel
        .basic_consume(
            DEAD_QUEUE.into(),
            "test-dlq".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let dead_message = tokio::time::timeout(Duration::from_secs(10), dead.next())
        .await?
        .expect("dead letter delivered")?;
    let actual: JobEnvelope = serde_json::from_slice(&dead_message.data)?;
    assert_eq!(actual.job_id, job.job_id);
    dead_message.ack(BasicAckOptions::default()).await?;
    connection.close(200, "test complete".into()).await?;
    Ok(())
}
