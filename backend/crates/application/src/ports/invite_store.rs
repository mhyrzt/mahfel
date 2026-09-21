use super::{BoxFuture, StoreError};
use mahfel_domain::{InviteCode, InviteLink, InviteLinkId, InviteTargetKind};

/// Persistence for revocable invite links (M4-13).
///
/// Codes are stored plaintext because redemption looks them up directly
/// (ADR 0021). Methods return [`BoxFuture`] so the port is dyn-compatible
/// (ADR 0023).
pub trait InviteStore: Send + Sync {
    /// [`StoreError::Conflict`] when the code is taken, which is what makes
    /// generation-with-retry possible.
    fn insert<'a>(&'a self, link: &'a InviteLink) -> BoxFuture<'a, Result<(), StoreError>>;

    /// The redemption path. Codes are matched on the canonical form.
    fn get_by_code(
        &self,
        code: InviteCode,
    ) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>>;

    fn get(&self, id: InviteLinkId) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>>;

    /// Every link for one target, newest first, revoked links included.
    fn list_for_target(
        &self,
        target_kind: InviteTargetKind,
        target_id: uuid::Uuid,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<InviteLink>, StoreError>>;

    /// Revokes only if not already revoked, so the first revocation time is
    /// never overwritten and a repeat is distinguishable.
    fn revoke_if_active(
        &self,
        id: InviteLinkId,
        revoked_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;

    /// Revoke `previous` and insert `replacement` in one transaction, so a
    /// reader never sees two live codes or none. A code collision rolls back.
    fn rotate<'a>(
        &'a self,
        previous: InviteLinkId,
        revoked_at: time::OffsetDateTime,
        replacement: &'a InviteLink,
    ) -> BoxFuture<'a, Result<(), StoreError>>;
}
