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
        Ok(Self { channel, stream })
    }

    pub async fn next(&mut self) -> Option<Result<JobDelivery, MessagingError>> {
        self.stream.next().await.map(|result| {
            result
                .map(|delivery| JobDelivery { delivery })
                .map_err(MessagingError::from)
        })
    }

    pub async fn stop(&self) -> Result<(), MessagingError> {
        self.channel
            .basic_cancel(self.stream.tag().clone(), Default::default())
            .await?;
        Ok(())
    }
}
