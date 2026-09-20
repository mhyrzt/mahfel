use crate::error::MediaClientError;
use mahfel_rtc_protocol::media::media_control_client::MediaControlClient;
use tonic::transport::{Channel, Endpoint};

#[derive(Clone)]
pub struct GrpcMediaGateway {
    pub(super) client: MediaControlClient<Channel>,
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

pub fn lazy(endpoint: String) -> Result<MediaControlClient<Channel>, MediaClientError> {
    Ok(MediaControlClient::new(
        Endpoint::from_shared(endpoint)?.connect_lazy(),
    ))
}
