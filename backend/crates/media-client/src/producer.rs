use super::gateway::GrpcMediaGateway;
use mahfel_domain::{ActorId, MediaKind, ProducerSource, RoomId};
use mahfel_rtc_protocol::media::{
    CloseParticipantRequest, ListProducersRequest, PauseProducerRequest, ProduceRequest,
    ResumeProducerRequest,
};

pub struct MediaProducer {
    pub producer_id: String,
    pub actor_id: String,
    pub kind: String,
    pub source: String,
}

impl GrpcMediaGateway {
    pub async fn list_producers(&self, room: RoomId) -> Result<Vec<MediaProducer>, tonic::Status> {
        let response = self
            .with_retry(|mut client| {
                let request = ListProducersRequest {
                    room_id: room.to_string(),
                };
                async move { client.list_producers(request).await }
            })
            .await?;
        Ok(response
            .producers
            .into_iter()
            .map(|entry| MediaProducer {
                producer_id: entry.producer_id,
                actor_id: entry.actor_id,
                kind: entry.kind,
                source: entry.source,
            })
            .collect())
    }

    pub async fn produce(
        &self,
        room: RoomId,
        participant_id: ActorId,
        transport_id: String,
        kind: MediaKind,
        source: ProducerSource,
        rtp_parameters: String,
    ) -> Result<String, tonic::Status> {
        let kind_str = kind.as_str().to_owned();

        let res = self
            .once(|mut client| {
                let req = ProduceRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.to_string(),
                    transport_id: transport_id.clone(),
                    kind: kind_str.clone(),
                    source: source.as_str().to_owned(),
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
