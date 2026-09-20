use crate::ports::{IdentityStore, ProvisionedIdentity};
use crate::users::{SyncIdentity, SyncIdentityError, sync_identity};
use std::{future::Future, pin::Pin};

/// Dyn-compatible form of the identity sync use case.
///
/// [`IdentityStore`] uses return-position `impl Future`, so it is not
/// dyn-compatible. Transports hold one provisioner behind `Arc<dyn ...>`
/// instead of making the router generic over the store.
pub trait IdentityProvisioner: Send + Sync {
    fn provision<'a>(
        &'a self,
        command: SyncIdentity,
    ) -> Pin<Box<dyn Future<Output = Result<ProvisionedIdentity, SyncIdentityError>> + Send + 'a>>;
}

impl<T> IdentityProvisioner for T
where
    T: IdentityStore,
{
    fn provision<'a>(
        &'a self,
        command: SyncIdentity,
    ) -> Pin<Box<dyn Future<Output = Result<ProvisionedIdentity, SyncIdentityError>> + Send + 'a>>
    {
        Box::pin(sync_identity(self, command))
    }
}
