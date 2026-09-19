use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidInput,
    Unprocessable,
    NotFound,
    Unauthorized,
    Forbidden,
    Conflict,
    Internal,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::Unprocessable => "unprocessable_entity",
            Self::NotFound => "not_found",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::Conflict => "conflict",
            Self::Internal => "internal_error",
        }
    }

    #[must_use]
    pub const fn status(self) -> StatusCode {
        match self {
            Self::InvalidInput => StatusCode::BAD_REQUEST,
            Self::Unprocessable => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

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
#[allow(dead_code)]
#[derive(Debug)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
    details: Option<Value>,
    status_override: Option<StatusCode>,
}

#[allow(dead_code)]
impl ApiError {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            status_override: None,
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

    #[must_use]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status_override = Some(status);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_override.unwrap_or_else(|| self.code.status());
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code.as_str().to_owned(),
                message: self.message,
                details: self.details,
            },
        };
        (status, Json(body)).into_response()
    }
}
