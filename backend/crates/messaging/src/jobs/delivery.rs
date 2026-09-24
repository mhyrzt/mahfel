use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions},
};

use super::MessagingError;

pub struct JobDelivery {
    pub(super) delivery: Delivery,
}

impl JobDelivery {
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.delivery.data
    }

    pub async fn ack(self) -> Result<(), MessagingError> {
        self.delivery.ack(BasicAckOptions::default()).await?;
        Ok(())
    }

    pub async fn dead_letter(self) -> Result<(), MessagingError> {
        self.delivery
            .nack(BasicNackOptions {
                requeue: false,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn requeue(self) -> Result<(), MessagingError> {
        self.delivery
            .nack(BasicNackOptions {
                requeue: true,
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}
