use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, ServicePrincipal};

/// Persistence for service principals and their API-key hashes.
///
/// Only the SHA-256 hash of an API key is ever stored; the raw key exists only
/// in the response that creates or rotates it. Methods return [`BoxFuture`] so
/// the port is dyn-compatible.
pub trait ServicePrincipalStore: Send + Sync {
    /// Persist a new principal, its actor row, and its first API-key hash in
    /// one transaction. Returns [`StoreError::Conflict`] when the name is taken.
    fn create<'a>(
        &'a self,
        principal: &'a ServicePrincipal,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>>;

    fn get(&self, id: ActorId) -> BoxFuture<'_, Result<Option<ServicePrincipal>, StoreError>>;

    fn list(&self) -> BoxFuture<'_, Result<Vec<ServicePrincipal>, StoreError>>;

    /// Mark the principal revoked and revoke its active keys.
    fn revoke(&self, id: ActorId) -> BoxFuture<'_, Result<(), StoreError>>;

    /// Revoke every active key and insert `api_key_hash` as the only active key,
    /// atomically and only while the principal is active.
    fn rotate_key<'a>(
        &'a self,
        id: ActorId,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Resolve the active principal for a presented key hash, if one exists.
    fn find_active_by_key_hash<'a>(
        &'a self,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<Option<ServicePrincipal>, StoreError>>;
}
