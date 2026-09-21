use super::StoreError;
use mahfel_domain::{ActorId, ServicePrincipal};

/// Persistence for service principals and their API-key hashes.
///
/// Only the SHA-256 hash of an API key is ever stored; the raw key exists only
/// in the response that creates or rotates it.
pub trait ServicePrincipalStore: Send + Sync {
    /// Persist a new principal, its actor row, and its first API-key hash in
    /// one transaction. Returns [`StoreError::Conflict`] when the name is taken.
    fn create(
        &self,
        principal: &ServicePrincipal,
        api_key_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;

    fn get(
        &self,
        id: ActorId,
    ) -> impl std::future::Future<Output = Result<Option<ServicePrincipal>, StoreError>> + Send;

    fn list(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<ServicePrincipal>, StoreError>> + Send;

    /// Mark the principal revoked and revoke its active keys.
    fn revoke(
        &self,
        id: ActorId,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;

    /// Revoke every active key and insert `api_key_hash` as the only active key,
    /// atomically and only while the principal is active.
    fn rotate_key(
        &self,
        id: ActorId,
        api_key_hash: &str,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;

    /// Resolve the active principal for a presented key hash, if one exists.
    fn find_active_by_key_hash(
        &self,
        api_key_hash: &str,
    ) -> impl std::future::Future<Output = Result<Option<ServicePrincipal>, StoreError>> + Send;
}
