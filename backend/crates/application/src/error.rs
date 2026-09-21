//! Transport-agnostic error classification.
//!
//! Application errors describe *what* failed; transports decide how to
//! represent it. Implementing [`ApplicationError`] lets a transport map an
//! error to a response without inspecting error strings.

/// Coarse, transport-agnostic failure category.
///
/// Variants name semantics, not HTTP codes; a transport maps them to a status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The request itself was malformed.
    InvalidInput,
    /// The value was well-formed but failed validation.
    Unprocessable,
    /// The target does not exist or is not visible to the caller.
    NotFound,
    /// Authentication is required or invalid.
    Unauthorized,
    /// Authenticated but not permitted.
    Forbidden,
    /// The operation conflicts with current state.
    Conflict,
    /// A required dependency is temporarily unavailable. Callers should
    /// degrade rather than treat the request as malformed. Messages must not
    /// reach clients.
    Unavailable,
    /// An unexpected internal failure. Messages must not reach clients.
    Internal,
}

/// An application error that can classify itself.
pub trait ApplicationError: std::error::Error + Send + Sync + 'static {
    /// Semantic category of this failure.
    fn kind(&self) -> ErrorKind;
}

/// Why an access guard rejected an operation.
///
/// Every guard collapses "the target does not exist" and "the caller may not
/// learn that it does" into [`GuardError::NotFound`], so a caller cannot probe
/// for the existence of resources it is not permitted to see. The variant set
/// is deliberately small: guards answer "may this caller proceed?", not "why
/// did the operation fail?".
///
/// Guards are shared across use cases, so each operation error converts a
/// denial into its own type with a small `From` impl.
#[derive(Debug, thiserror::Error)]
pub(crate) enum GuardError {
    #[error("not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("storage failure: {0}")]
    Store(#[from] crate::ports::StoreError),
}
