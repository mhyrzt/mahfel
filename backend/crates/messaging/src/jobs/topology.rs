use lapin::{
    Connection, ExchangeKind,
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
};

use crate::{DEAD_EXCHANGE, DEAD_QUEUE, FILE_QUEUE, FILE_ROUTING_KEY, JOB_EXCHANGE};

use super::MessagingError;

/// Safe to call again on every reconnect. Queue property changes require a new queue name.
pub async fn declare_topology(connection: &Connection) -> Result<(), MessagingError> {
    let channel = connection.create_channel().await?;
    channel
        .exchange_declare(
            JOB_EXCHANGE.into(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .exchange_declare(
            DEAD_EXCHANGE.into(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    let mut args = FieldTable::default();
    args.insert(
        "x-queue-type".into(),
        AMQPValue::LongString("quorum".into()),
    );
    args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString(DEAD_EXCHANGE.into()),
    );
    args.insert(
        "x-dead-letter-routing-key".into(),
        AMQPValue::LongString(FILE_ROUTING_KEY.into()),
    );
    channel
        .queue_declare(
            FILE_QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            args,
        )
        .await?;
    channel
        .queue_bind(
            FILE_QUEUE.into(),
            JOB_EXCHANGE.into(),
            FILE_ROUTING_KEY.into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut dead_args = FieldTable::default();
    dead_args.insert(
        "x-queue-type".into(),
        AMQPValue::LongString("quorum".into()),
    );
    dead_args.insert("x-message-ttl".into(), AMQPValue::LongInt(604_800_000));
    dead_args.insert(
        "x-overflow".into(),
        AMQPValue::LongString("reject-publish".into()),
    );
    dead_args.insert(
        "x-max-length-bytes".into(),
        AMQPValue::LongLongInt(104_857_600),
    );
    channel
        .queue_declare(
            DEAD_QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            dead_args,
        )
        .await?;
    channel
        .queue_bind(
            DEAD_QUEUE.into(),
            DEAD_EXCHANGE.into(),
            FILE_ROUTING_KEY.into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    channel.close(200, "topology ready".into()).await?;
    Ok(())
}
