use super::gateway::GrpcMediaGateway;
use mahfel_domain::{ActorId, RoomId};
use mahfel_rtc_protocol::media::{ConnectWebRtcTransportRequest, CreateWebRtcTransportRequest};

impl GrpcMediaGateway {
    pub async fn create_transport(
        &self,
        room: RoomId,
        participant_id: ActorId,
    ) -> Result<(String, String, String, String), tonic::Status> {
        let res = self
            .with_retry(|mut client| {
                let req = CreateWebRtcTransportRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.to_string(),
                };
                async move { client.create_web_rtc_transport(req).await }
            })
            .await?;
        Ok((
            res.transport_id,
            res.ice_parameters,
            res.ice_candidates,
            res.dtls_parameters,
        ))
    }

    pub async fn connect_transport(
        &self,
        room: RoomId,
        transport_id: String,
        dtls_parameters: String,
    ) -> Result<(), tonic::Status> {
        self.with_retry(|mut client| {
            let req = ConnectWebRtcTransportRequest {
                room_id: room.to_string(),
                transport_id: transport_id.clone(),
                dtls_parameters: dtls_parameters.clone(),
            };
            async move { client.connect_web_rtc_transport(req).await }
        })
        .await
        .map(|_| ())
    }
}
