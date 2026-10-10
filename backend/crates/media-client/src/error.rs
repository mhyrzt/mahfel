use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaClientError {
    #[error("invalid media control credential")]
    Credential,
    #[error("invalid media service endpoint: {0}")]
    Endpoint(#[from] tonic::transport::Error),
}
