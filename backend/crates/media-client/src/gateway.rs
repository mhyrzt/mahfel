use crate::error::MediaClientError;
use mahfel_rtc_protocol::media::media_control_service_client::MediaControlServiceClient;
use mahfel_rtc_protocol::media::{WatchActiveSpeakersRequest, WatchActiveSpeakersResponse};
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

    pub async fn watch_active_speakers(
        &self,
    ) -> Result<tonic::Streaming<WatchActiveSpeakersResponse>, tonic::Status> {
        let mut client = self.client.clone();
        client
            .watch_active_speakers(WatchActiveSpeakersRequest {})
            .await
            .map(tonic::Response::into_inner)
    }
}

pub fn lazy(endpoint: String) -> Result<MediaControlServiceClient<Channel>, MediaClientError> {
    Ok(MediaControlServiceClient::new(
        Endpoint::from_shared(endpoint)?.connect_lazy(),
    ))
}
