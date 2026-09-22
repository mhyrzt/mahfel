use mahfel_domain::RoomId;
use mediasoup::worker::CreateRouterError;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RtcError {
    #[error("worker count must be greater than zero")]
    NoWorkers,
    #[error("RTC port range {minimum}-{maximum} cannot provide {workers} worker ports")]
    InsufficientPorts {
        minimum: u16,
        maximum: u16,
        workers: usize,
    },
    #[error("invalid media listen address: {0}")]
    InvalidListenAddress(#[from] std::net::AddrParseError),
    #[error("mediasoup worker initialization failed: {0}")]
    Worker(#[from] std::io::Error),
    #[error("WebRTC server initialization failed: {0}")]
    WebRtcServer(#[from] mediasoup::worker::CreateWebRtcServerError),
    #[error("router creation failed: {0}")]
    Router(#[from] CreateRouterError),
    #[error("audio observer failed: {0}")]
    Observer(String),
    #[error("room not found: {room_id}")]
    RoomNotFound { room_id: RoomId },
    #[error("transport not found: {transport_id}")]
    TransportNotFound { transport_id: String },
    #[error("transport creation failed: {0}")]
    Transport(String),
    #[error("transport connect failed: {0}")]
    Connect(String),
    #[error("invalid DTLS parameters: {0}")]
    Dtls(String),
    #[error("invalid RTP parameters: {0}")]
    RtpParameters(String),
    #[error("producer creation failed: {0}")]
    Producer(String),
    #[error("producer not found: {producer_id}")]
    ProducerNotFound { producer_id: String },
    #[error("invalid RTP capabilities: {0}")]
    RtpCapabilities(String),
    #[error("consumer creation failed: {0}")]
    Consumer(String),
    #[error("consumer not found: {consumer_id}")]
    ConsumerNotFound { consumer_id: String },
    #[error("participant mismatch")]
    ParticipantMismatch,
}

#[allow(dead_code)]
impl RtcError {
    pub(crate) fn room_not_found(room_id: RoomId) -> Self {
        Self::RoomNotFound { room_id }
    }
}

impl From<RtcError> for tonic::Status {
    fn from(e: RtcError) -> Self {
        use RtcError::*;
        match e {
            RoomNotFound { .. }
            | TransportNotFound { .. }
            | ProducerNotFound { .. }
            | ConsumerNotFound { .. } => Self::not_found(e.to_string()),
            Dtls(_) | RtpParameters(_) | RtpCapabilities(_) => {
                Self::invalid_argument(e.to_string())
            }
            _ => Self::internal(e.to_string()),
        }
    }
}
