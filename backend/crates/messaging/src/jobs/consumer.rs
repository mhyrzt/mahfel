use futures_util::StreamExt;
use lapin::{
    Channel, Connection,
    options::{BasicConsumeOptions, BasicQosOptions},
    types::FieldTable,
};

use crate::FILE_QUEUE;

use super::{JobDelivery, MessagingError};

pub struct JobConsumer {
    channel: Channel,
    stream: lapin::Consumer,
    recording_stream: Option<lapin::Consumer>,
}

impl JobConsumer {
    pub async fn new(connection: &Connection, prefetch: u16) -> Result<Self, MessagingError> {
        let channel = connection.create_channel().await?;
        channel
            .basic_qos(prefetch, BasicQosOptions::default())
            .await?;
        let stream = channel
            .basic_consume(
                FILE_QUEUE.into(),
                "mahfel-worker".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let recording_stream = channel
            .basic_consume(
                crate::RECORDING_QUEUE.into(),
                "mahfel-recording-worker".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(Self {
            channel,
            stream,
            recording_stream: Some(recording_stream),
        })
    }

    pub async fn new_export(
        connection: &Connection,
        prefetch: u16,
    ) -> Result<Self, MessagingError> {
        let channel = connection.create_channel().await?;
        channel
            .basic_qos(prefetch, BasicQosOptions::default())
            .await?;
        let stream = channel
            .basic_consume(
                crate::EXPORT_QUEUE.into(),
                "mahfel-export-worker".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(Self {
            channel,
            stream,
            recording_stream: None,
        })
    }

    pub async fn next(&mut self) -> Option<Result<JobDelivery, MessagingError>> {
        let next = if let Some(recording_stream) = &mut self.recording_stream {
            tokio::select! { next = self.stream.next() => next, next = recording_stream.next() => next }
        } else {
            self.stream.next().await
        };
        next.map(|result| {
            result
                .map(|delivery| JobDelivery { delivery })
                .map_err(MessagingError::from)
        })
    }

    pub async fn stop(&self) -> Result<(), MessagingError> {
        if let Some(stream) = &self.recording_stream {
            self.channel
                .basic_cancel(stream.tag().clone(), Default::default())
                .await?;
        }
        self.channel
            .basic_cancel(self.stream.tag().clone(), Default::default())
            .await?;
        Ok(())
    }
}
