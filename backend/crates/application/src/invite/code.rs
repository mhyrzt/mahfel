//! Generate a code that is not already taken.
//!
//! Uniqueness is the store's job: the caller proposes a code and the unique
//! index decides, so two replicas generating at the same instant cannot both
//! win. Collisions are not expected at 26^10; the bound exists so the failure
//! mode is a deliberate error instead of an unbounded loop.

use super::error::InviteOperationError;
use crate::ports::{InviteStore, StoreError};
use mahfel_domain::{InviteCode, InviteLink};

/// Small on purpose: repeated collisions mean the store is not enforcing
/// uniqueness, which is a defect rather than bad luck.
const MAX_ATTEMPTS: usize = 5;

/// Insert `link` with a freshly generated code, retrying a code collision.
///
/// # Errors
/// [`InviteOperationError::CodeExhausted`] when every attempt collided.
pub async fn insert_with_unique_code<S>(
    invites: &S,
    link: InviteLink,
) -> Result<InviteLink, InviteOperationError>
where
    S: InviteStore + ?Sized,
{
    for _ in 0..MAX_ATTEMPTS {
        let candidate = InviteLink {
            code: InviteCode::generate(),
            ..link.clone()
        };
        match invites.insert(&candidate).await {
            Ok(()) => return Ok(candidate),
            Err(StoreError::Conflict(_)) => continue,
            Err(other) => return Err(other.into()),
        }
    }
    Err(InviteOperationError::CodeExhausted)
}
