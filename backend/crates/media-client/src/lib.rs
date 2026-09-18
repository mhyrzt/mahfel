use mahfel_application::ports::{MediaGateway, MediaGatewayError};
use mahfel_domain::RoomId;
use mahfel_rtc_protocol::media::media_control_client::MediaControlClient;
use thiserror::Error;
use tonic::transport::{Channel, Endpoint};

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
