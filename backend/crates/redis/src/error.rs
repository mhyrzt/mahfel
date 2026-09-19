use mahfel_application::ports::CoordinationError;
use thiserror::Error;

/// Redis adapter failure.
///
/// Connection and timeout failures are `Unavailable` so callers can degrade;
/// everything else is an unexpected coordination failure.
#[derive(Debug, Error)]
pub enum RedisError {
    #[error("redis unavailable: {0}")]
    Unavailable(String),
    #[error("redis failure: {0}")]
    Unexpected(String),
}

impl From<redis::RedisError> for RedisError {
    fn from(error: redis::RedisError) -> Self {
        if error.is_timeout()
            || error.is_connection_dropped()
            || error.is_connection_refusal()
            || error.is_io_error()
        {
            Self::Unavailable(error.to_string())
        } else {
            Self::Unexpected(error.to_string())
        }
    }
}

impl From<RedisError> for CoordinationError {
    fn from(error: RedisError) -> Self {
        match error {
            RedisError::Unavailable(message) => Self::Unavailable(message),
            RedisError::Unexpected(message) => Self::Unexpected(message),
        }
    }
}
