use super::gateway::GrpcMediaGateway;
use mahfel_domain::{ActorId, RoomId};
use mahfel_rtc_protocol::media::{ConsumeRequest, PauseConsumerRequest, ResumeConsumerRequest};

impl GrpcMediaGateway {
    pub async fn consume(
        &self,
        room: RoomId,
        participant_id: ActorId,
        producer_id: String,
        transport_id: String,
        rtp_capabilities: String,
    ) -> Result<(String, String, bool), tonic::Status> {
        let res = self
            .once(|mut client| {
                let req = ConsumeRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.to_string(),
                    producer_id: producer_id.clone(),
                    transport_id: transport_id.clone(),
                    rtp_capabilities: rtp_capabilities.clone(),
                };
                async move { client.consume(req).await }
            })
            .await?;
        Ok((res.consumer_id, res.rtp_parameters, res.paused))
    }

    pub async fn pause_consumer(
        &self,
        room: RoomId,
        consumer_id: String,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = PauseConsumerRequest {
                room_id: room.to_string(),
                consumer_id: consumer_id.clone(),
            };
            async move { client.pause_consumer(req).await }
        })
        .await
        .map(|_| ())
    }

    pub async fn resume_consumer(
        &self,
        room: RoomId,
        consumer_id: String,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = ResumeConsumerRequest {
                room_id: room.to_string(),
                consumer_id: consumer_id.clone(),
            };
            async move { client.resume_consumer(req).await }
        })
        .await
        .map(|_| ())
    }
}
