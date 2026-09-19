/// Implement `From<AppError> for ApiError` for each application error type,
/// using its [`ApplicationError::kind`](mahfel_application::ApplicationError).
///
/// This gives handlers bare `?` on use-case results. `From` is implemented per
/// error type because a blanket `impl<T: ApplicationError> From<T> for ApiError`
/// would overlap with the standard reflexive `impl<T> From<T> for T`.
macro_rules! api_error_from {
    ($($error:ty),+ $(,)?) => {$(
        impl From<$error> for $crate::http::error::ApiError {
            fn from(error: $error) -> Self {
                $crate::http::error::ApiError::from_application(
                    ::mahfel_application::ApplicationError::kind(&error),
                    error.to_string(),
                )
            }
        }
    )+};
}

pub(crate) use api_error_from;
