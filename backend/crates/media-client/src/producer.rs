use super::gateway::GrpcMediaGateway;
use mahfel_domain::{ActorId, RoomId};
use mahfel_rtc_protocol::media::{
    CloseParticipantRequest, PauseProducerRequest, ProduceRequest, ResumeProducerRequest,
};
use mediasoup::prelude::MediaKind;

impl GrpcMediaGateway {
    pub async fn produce(
        &self,
        room: RoomId,
        participant_id: ActorId,
        transport_id: String,
        kind: MediaKind,
        rtp_parameters: String,
    ) -> Result<String, tonic::Status> {
        let kind_str = match kind {
            MediaKind::Audio => "audio",
            MediaKind::Video => "video",
        }
        .to_owned();

        let res = self
            .with_retry(|mut client| {
                let req = ProduceRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.to_string(),
                    transport_id: transport_id.clone(),
                    kind: kind_str.clone(),
                    rtp_parameters: rtp_parameters.clone(),
                };
                async move { client.produce(req).await }
            })
            .await?;
        Ok(res.producer_id)
    }

    /// Pause one producer without destroying it.
    ///
    /// Pausing is what lets an enforced mute map onto media state one-to-one:
    /// resuming needs no renegotiation, and the caller's durable mute record
    /// never points at a producer that no longer exists.
    pub async fn pause_producer(
        &self,
        room: RoomId,
        producer_id: String,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = PauseProducerRequest {
                room_id: room.to_string(),
                producer_id: producer_id.clone(),
            };
            async move { client.pause_producer(req).await }
        })
        .await
        .map(|_| ())
    }

    pub async fn resume_producer(
        &self,
        room: RoomId,
        producer_id: String,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = ResumeProducerRequest {
                room_id: room.to_string(),
                producer_id: producer_id.clone(),
            };
            async move { client.resume_producer(req).await }
        })
        .await
        .map(|_| ())
    }

    /// Close every producer one participant owns.
    ///
    /// Removal is terminal, so closing is correct rather than pausing. Succeeds
    /// when the participant published nothing, so removing a silent participant
    /// is not an error.
    pub async fn close_participant(
        &self,
        room: RoomId,
        participant_id: ActorId,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = CloseParticipantRequest {
                room_id: room.to_string(),
                participant_id: participant_id.to_string(),
            };
            async move { client.close_participant(req).await }
        })
        .await
        .map(|_| ())
    }
}
