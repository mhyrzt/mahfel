use super::gateway::GrpcMediaGateway;
use mahfel_domain::{ActorId, RoomId};
use mahfel_rtc_protocol::media::ProduceRequest;
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
}
