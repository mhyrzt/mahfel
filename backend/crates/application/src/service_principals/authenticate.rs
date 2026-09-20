use super::api_key;
use crate::ports::{ServicePrincipalStore, StoreError};
use mahfel_domain::ServicePrincipal;

/// Resolve the active service principal for a presented raw API key.
///
/// Returns `Ok(None)` when the key is unknown or belongs to a revoked key or
/// principal. The presented key is hashed first, so a raw key is never compared
/// or stored directly.
pub async fn authenticate_service_principal<S>(
    store: &S,
    presented_key: &str,
) -> Result<Option<ServicePrincipal>, StoreError>
where
    S: ServicePrincipalStore,
{
    store
        .find_active_by_key_hash(&api_key::hash(presented_key))
        .await
}
