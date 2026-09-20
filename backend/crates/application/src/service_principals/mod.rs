//! Service principal use cases: provisioning, rotation, revocation,
//! authentication, and listing.

pub mod api_key;
mod authenticate;
mod create;
mod list;
mod revoke;
mod rotate;

pub use authenticate::authenticate_service_principal;
pub use create::{
    CreateServicePrincipal, CreateServicePrincipalError, CreatedServicePrincipal,
    create_service_principal,
};
pub use list::{ListServicePrincipalsError, list_service_principals};
pub use revoke::{RevokeServicePrincipal, RevokeServicePrincipalError, revoke_service_principal};
pub use rotate::{
    RotateServicePrincipalKey, RotateServicePrincipalKeyError, RotatedServicePrincipalKey,
    rotate_service_principal_key,
};
