use lapin::{
    BasicProperties, Channel, Confirmation, Connection,
    options::BasicPublishOptions,
    types::{AMQPValue, FieldTable},
};
use mahfel_application::jobs::JobEnvelope;
use std::time::Duration;

use crate::JOB_EXCHANGE;

use super::MessagingError;

pub struct ConfirmedPublisher {
    channel: Channel,
}

impl ConfirmedPublisher {
    pub async fn new(connection: &Connection) -> Result<Self, MessagingError> {
        let channel = connection.create_channel().await?;
        channel.confirm_select(Default::default()).await?;
        Ok(Self { channel })
    }

    pub async fn publish(&self, job: &JobEnvelope) -> Result<(), MessagingError> {
        let payload = serde_json::to_vec(job)?;
        let mut headers = FieldTable::default();
        if let Some(traceparent) = &job.traceparent {
            headers.insert(
                "traceparent".into(),
                AMQPValue::LongString(traceparent.clone().into()),
            );
        }
        if let Some(tracestate) = &job.tracestate {
            headers.insert(
                "tracestate".into(),
                AMQPValue::LongString(tracestate.clone().into()),
            );
        }
        let confirm = self
            .channel
            .basic_publish(
                JOB_EXCHANGE.into(),
                job.routing_key().into(),
                BasicPublishOptions {
                    mandatory: true,
                    ..Default::default()
                },
                &payload,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2)
                    .with_message_id(job.job_id.to_string().into())
                    .with_correlation_id(job.correlation_id.as_deref().unwrap_or("").into())
                    .with_headers(headers),
            )
            .await?;
        let result = tokio::time::timeout(Duration::from_secs(5), confirm)
            .await
            .map_err(|_| MessagingError::Timeout)??;
        match result {
            Confirmation::Ack(None) => Ok(()),
            Confirmation::Ack(Some(_)) => Err(MessagingError::Unroutable),
            Confirmation::Nack(_) => Err(MessagingError::Rejected),
            Confirmation::NotRequested => Err(MessagingError::NoConfirm),
        }
    }
}
