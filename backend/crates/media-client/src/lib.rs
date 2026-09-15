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
