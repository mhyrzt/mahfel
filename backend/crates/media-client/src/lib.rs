use mahfel_application::ports::{MediaGateway, MediaGatewayError};
use mahfel_domain::RoomId;
use mahfel_rtc_protocol::media::{
    ConnectWebRtcTransportRequest, ConsumeRequest, CreateWebRtcTransportRequest,
    PauseConsumerRequest, ProduceRequest, ResumeConsumerRequest,
    media_control_client::MediaControlClient,
};
use thiserror::Error;
use tonic::{
    Code,
    transport::{Channel, Endpoint},
};

#[derive(Debug, Error)]
pub enum MediaClientError {
    #[error("invalid media service endpoint: {0}")]
    Endpoint(#[from] tonic::transport::Error),
}

pub fn lazy(endpoint: String) -> Result<MediaControlClient<Channel>, MediaClientError> {
    Ok(MediaControlClient::new(
        Endpoint::from_shared(endpoint)?.connect_lazy(),
    ))
}

/// Adapter that hides Tonic/`Channel` behind the application `MediaGateway` port.
///
/// Transports depend only on `mahfel-application`; `mahfel-api` builds this
/// adapter at the composition root and injects it as `&dyn MediaGateway`.
#[derive(Clone)]
pub struct GrpcMediaGateway {
    client: MediaControlClient<Channel>,
}

impl GrpcMediaGateway {
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: MediaControlClient::new(channel),
        }
    }

    pub fn lazy(endpoint: String) -> Result<Self, MediaClientError> {
        Ok(Self::from_channel(
            Endpoint::from_shared(endpoint)?.connect_lazy(),
        ))
    }

    #[must_use]
    pub fn into_inner(self) -> MediaControlClient<Channel> {
        self.client
    }

    async fn with_retry<F, T, Fut>(&self, mut operation: F) -> Result<T, tonic::Status>
    where
        F: FnMut(MediaControlClient<Channel>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<tonic::Response<T>, tonic::Status>> + Send,
        T: Send,
    {
        let mut last_status: Option<tonic::Status> = None;
        for attempt in 0..3 {
            let client = self.client.clone();
            let fut = operation(client);
            let res = tokio::time::timeout(std::time::Duration::from_secs(5), fut).await;
            match res {
                Ok(Ok(response)) => return Ok(response.into_inner()),
                Ok(Err(status)) => {
                    if status.code() == Code::Unavailable && attempt < 2 {
                        last_status = Some(status);
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempt)))
                            .await;
                        continue;
                    }
                    return Err(status);
                }
                Err(_) => {
                    let status = tonic::Status::deadline_exceeded("media gateway timeout");
                    if attempt < 2 {
                        last_status = Some(status);
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempt)))
                            .await;
                        continue;
                    }
                    return Err(status);
                }
            }
        }
        Err(last_status.unwrap_or_else(|| tonic::Status::unavailable("media unavailable")))
    }

    pub async fn create_transport(
        &self,
        room: RoomId,
        participant_id: String,
    ) -> Result<(String, String, String, String), tonic::Status> {
        let res = self
            .with_retry(|mut client| {
                let req = CreateWebRtcTransportRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.clone(),
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

    pub async fn produce(
        &self,
        room: RoomId,
        participant_id: String,
        transport_id: String,
        kind: String,
        rtp_parameters: String,
    ) -> Result<String, tonic::Status> {
        let res = self
            .with_retry(|mut client| {
                let req = ProduceRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.clone(),
                    transport_id: transport_id.clone(),
                    kind: kind.clone(),
                    rtp_parameters: rtp_parameters.clone(),
                };
                async move { client.produce(req).await }
            })
            .await?;
        Ok(res.producer_id)
    }

    pub async fn consume(
        &self,
        room: RoomId,
        participant_id: String,
        producer_id: String,
        transport_id: String,
        rtp_capabilities: String,
    ) -> Result<(String, String, bool), tonic::Status> {
        let res = self
            .with_retry(|mut client| {
                let req = ConsumeRequest {
                    room_id: room.to_string(),
                    participant_id: participant_id.clone(),
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

impl MediaGateway for GrpcMediaGateway {
    async fn create_room(&self, room: RoomId) -> Result<(), MediaGatewayError> {
        let mut client = self.client.clone();
        let request = mahfel_rtc_protocol::media::CreateRoomRequest {
            room_id: room.to_string(),
        };
        client
            .create_room(request)
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
    }

    async fn close_room(&self, room: RoomId) -> Result<(), MediaGatewayError> {
        let mut client = self.client.clone();
        let request = mahfel_rtc_protocol::media::DeleteRoomRequest {
            room_id: room.to_string(),
        };
        client
            .delete_room(request)
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
    }
}
