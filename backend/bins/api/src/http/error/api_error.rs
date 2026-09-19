use super::code::ErrorCode;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use mahfel_application::ErrorKind;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

/// Canonical API error that always renders as the JSON error envelope.
///
/// Validation and known domain failures must not become 500. Correlation is
/// carried by the `x-request-id` response header set by the request-id layer.
#[derive(Debug)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
    details: Option<Value>,
}

impl ApiError {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    #[must_use]
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unprocessable, message)
    }

    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, message)
    }

    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, message)
    }

    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Render an application error from its semantic kind.
    ///
    /// `Internal` never exposes the underlying message: storage and
    /// infrastructure details must not reach clients.
    #[must_use]
    pub fn from_application(kind: ErrorKind, message: String) -> Self {
        match kind {
            ErrorKind::InvalidInput => Self::bad_request(message),
            ErrorKind::Unprocessable => Self::unprocessable(message),
            ErrorKind::NotFound => Self::not_found(message),
            ErrorKind::Unauthorized => Self::unauthorized(message),
            ErrorKind::Forbidden => Self::forbidden(message),
            ErrorKind::Conflict => Self::new(ErrorCode::Conflict, message),
            ErrorKind::Internal => Self::internal("internal error"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code.as_str().to_owned(),
                message: self.message,
                details: self.details,
            },
        };
        (self.code.status(), Json(body)).into_response()
    }
}
