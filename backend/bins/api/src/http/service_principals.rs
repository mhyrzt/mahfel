//! Transport seam for service principal administration.
//!
//! [`ServicePrincipalStore`] uses return-position `impl Future`, so it is not
//! dyn-compatible. Rather than thread a second generic parameter through every
//! route, the admin surface is bound behind this small dyn-compatible trait,
//! matching the existing [`crate::http::auth::RequestAuthenticator`] seam.

use futures_util::future::BoxFuture;
use mahfel_application::ports::ServicePrincipalStore;
use mahfel_application::service_principals::{
    CreateServicePrincipal, CreateServicePrincipalError, CreatedServicePrincipal,
    ListServicePrincipalsError, RevokeServicePrincipal, RevokeServicePrincipalError,
    RotateServicePrincipalKey, RotateServicePrincipalKeyError, RotatedServicePrincipalKey,
    create_service_principal, list_service_principals, revoke_service_principal,
    rotate_service_principal_key,
};
use mahfel_domain::{ActorId, ServicePrincipal};

/// Administrative operations on service principals.
pub trait ServicePrincipalAdmin: Send + Sync + 'static {
    fn create<'a>(
        &'a self,
        command: CreateServicePrincipal,
    ) -> BoxFuture<'a, Result<CreatedServicePrincipal, CreateServicePrincipalError>>;

    fn list<'a>(
        &'a self,
    ) -> BoxFuture<'a, Result<Vec<ServicePrincipal>, ListServicePrincipalsError>>;

    fn rotate<'a>(
        &'a self,
        command: RotateServicePrincipalKey,
    ) -> BoxFuture<'a, Result<RotatedServicePrincipalKey, RotateServicePrincipalKeyError>>;

    fn revoke<'a>(&'a self, id: ActorId) -> BoxFuture<'a, Result<(), RevokeServicePrincipalError>>;
}

/// [`ServicePrincipalAdmin`] backed by a [`ServicePrincipalStore`].
#[derive(Clone)]
pub struct StoreServicePrincipalAdmin<S> {
    store: S,
}

impl<S> StoreServicePrincipalAdmin<S> {
    #[must_use]
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> ServicePrincipalAdmin for StoreServicePrincipalAdmin<S>
where
    S: ServicePrincipalStore + 'static,
{
    fn create<'a>(
        &'a self,
        command: CreateServicePrincipal,
    ) -> BoxFuture<'a, Result<CreatedServicePrincipal, CreateServicePrincipalError>> {
        Box::pin(create_service_principal(&self.store, command))
    }

    fn list<'a>(
        &'a self,
    ) -> BoxFuture<'a, Result<Vec<ServicePrincipal>, ListServicePrincipalsError>> {
        Box::pin(list_service_principals(&self.store))
    }

    fn rotate<'a>(
        &'a self,
        command: RotateServicePrincipalKey,
    ) -> BoxFuture<'a, Result<RotatedServicePrincipalKey, RotateServicePrincipalKeyError>> {
        Box::pin(rotate_service_principal_key(&self.store, command))
    }

    fn revoke<'a>(&'a self, id: ActorId) -> BoxFuture<'a, Result<(), RevokeServicePrincipalError>> {
        let command = RevokeServicePrincipal { id };
        Box::pin(revoke_service_principal(&self.store, command))
    }
}
