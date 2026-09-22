use crate::error::MediaClientError;
use mahfel_rtc_protocol::media::media_control_service_client::MediaControlServiceClient;
use tonic::transport::{Channel, Endpoint};

#[derive(Clone)]
pub struct GrpcMediaGateway {
    pub(super) client: MediaControlServiceClient<Channel>,
}

impl GrpcMediaGateway {
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: MediaControlServiceClient::new(channel),
        }
    }

    pub fn lazy(endpoint: String) -> Result<Self, MediaClientError> {
        Ok(Self::from_channel(
            Endpoint::from_shared(endpoint)?.connect_lazy(),
        ))
    }

    #[must_use]
    pub fn into_inner(self) -> MediaControlServiceClient<Channel> {
        self.client
    }
}

pub fn lazy(endpoint: String) -> Result<MediaControlServiceClient<Channel>, MediaClientError> {
    Ok(MediaControlServiceClient::new(
        Endpoint::from_shared(endpoint)?.connect_lazy(),
    ))
}
