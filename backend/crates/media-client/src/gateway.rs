use crate::error::MediaClientError;
use mahfel_rtc_protocol::media::media_control_service_client::MediaControlServiceClient;
use mahfel_rtc_protocol::media::{WatchActiveSpeakersRequest, WatchActiveSpeakersResponse};
use tonic::transport::{Channel, Endpoint};
use tonic::{
    Request, Status,
    metadata::{Ascii, MetadataValue},
    service::{Interceptor, interceptor::InterceptedService},
};

#[derive(Clone)]
pub struct MediaAuthorization(Option<MetadataValue<Ascii>>);

impl Interceptor for MediaAuthorization {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(value) = &self.0 {
            request
                .metadata_mut()
                .insert("authorization", value.clone());
        }
        Ok(request)
    }
}

pub type MediaControlClient =
    MediaControlServiceClient<InterceptedService<Channel, MediaAuthorization>>;

#[derive(Clone)]
pub struct GrpcMediaGateway {
    pub(super) client: MediaControlClient,
}

impl GrpcMediaGateway {
    pub async fn prepare_recording(&self, room_id: String) -> Result<(), tonic::Status> {
        let mut client = self.client.clone();
        let mut request =
            tonic::Request::new(mahfel_rtc_protocol::media::PrepareRecordingRequest { room_id });
        request.set_timeout(std::time::Duration::from_secs(10));
        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.prepare_recording(request),
        )
        .await
        .map_err(|_| tonic::Status::deadline_exceeded("recording preparation deadline"))??;
        Ok(())
    }

    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: MediaControlServiceClient::with_interceptor(channel, MediaAuthorization(None)),
        }
    }

    pub fn authenticated(endpoint: String, token: &str) -> Result<Self, MediaClientError> {
        if token.len() < 32 {
            return Err(MediaClientError::Credential);
        }
        let value = format!("Bearer {token}")
            .parse()
            .map_err(|_| MediaClientError::Credential)?;
        let channel = Endpoint::from_shared(endpoint)?.connect_lazy();
        Ok(Self {
            client: MediaControlServiceClient::with_interceptor(
                channel,
                MediaAuthorization(Some(value)),
            ),
        })
    }

    pub fn lazy(endpoint: String) -> Result<Self, MediaClientError> {
        Ok(Self::from_channel(
            Endpoint::from_shared(endpoint)?.connect_lazy(),
        ))
    }

    #[must_use]
    pub fn into_inner(self) -> MediaControlClient {
        self.client
    }

    pub async fn reconcile_recording(
        &self,
        request: mahfel_rtc_protocol::media::ReconcileRecordingRequest,
    ) -> Result<mahfel_rtc_protocol::media::ReconcileRecordingResponse, tonic::Status> {
        let mut client = self.client.clone();
        let mut request = tonic::Request::new(request);
        request.set_timeout(std::time::Duration::from_secs(5));
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.reconcile_recording(request),
        )
        .await
        .map_err(|_| tonic::Status::deadline_exceeded("recording control deadline"))?
        .map(tonic::Response::into_inner)
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
