use super::gateway::GrpcMediaGateway;
use mahfel_application::ports::{BoxFuture, MediaGateway, MediaGatewayError};
use mahfel_domain::{ActorId, ProducerSource, RoomId};
use mahfel_rtc_protocol::media::{
    ClearRoomMediaRequest, CloseParticipantSourceRequest, CreateRoomRequest, DeleteRoomRequest,
    PauseParticipantSourceRequest, ResumeParticipantSourceRequest,
};

impl GrpcMediaGateway {
    pub async fn clear_room_media(&self, room: RoomId) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| async move {
            client
                .clear_room_media(ClearRoomMediaRequest {
                    room_id: room.to_string(),
                })
                .await
        })
        .await
        .map(|_| ())
    }
}

impl MediaGateway for GrpcMediaGateway {
    fn create_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            self.once(|mut client| async move {
                client
                    .create_room(CreateRoomRequest {
                        room_id: room.to_string(),
                    })
                    .await
            })
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
        })
    }

    fn close_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            self.once(|mut client| async move {
                client
                    .delete_room(DeleteRoomRequest {
                        room_id: room.to_string(),
                    })
                    .await
            })
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
        })
    }

    fn pause_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        let producer_id = producer_id.to_owned();
        Box::pin(async move {
            self.pause_producer(room, producer_id)
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))
        })
    }

    fn resume_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        let producer_id = producer_id.to_owned();
        Box::pin(async move {
            self.resume_producer(room, producer_id)
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))
        })
    }

    fn close_participant(
        &self,
        room: RoomId,
        participant_id: ActorId,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            self.close_participant(room, participant_id)
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))
        })
    }

    fn close_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            let response = self
                .once(|mut client| async move {
                    client
                        .close_participant_source(CloseParticipantSourceRequest {
                            room_id: room.to_string(),
                            participant_id: participant_id.to_string(),
                            source: source.as_str().to_owned(),
                        })
                        .await
                })
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))?;
            Ok(response.closed as usize)
        })
    }

    fn pause_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            let response = self
                .once(|mut client| async move {
                    client
                        .pause_participant_source(PauseParticipantSourceRequest {
                            room_id: room.to_string(),
                            participant_id: participant_id.to_string(),
                            source: source.as_str().to_owned(),
                        })
                        .await
                })
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))?;
            Ok(response.affected as usize)
        })
    }

    fn resume_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            let response = self
                .once(|mut client| async move {
                    client
                        .resume_participant_source(ResumeParticipantSourceRequest {
                            room_id: room.to_string(),
                            participant_id: participant_id.to_string(),
                            source: source.as_str().to_owned(),
                        })
                        .await
                })
                .await
                .map_err(|e| MediaGatewayError::transport(e.to_string()))?;
            Ok(response.affected as usize)
        })
    }
}
