use mahfel_application::service_principals::{CreatedServicePrincipal, RotatedServicePrincipalKey};
use mahfel_domain::ServicePrincipal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateServicePrincipalRequest {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ServicePrincipalResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) display_name: Option<String>,
    pub(crate) status: String,
}

impl From<&ServicePrincipal> for ServicePrincipalResponse {
    fn from(principal: &ServicePrincipal) -> Self {
        Self {
            id: principal.id.to_string(),
            name: principal.name.to_string(),
            display_name: principal.display_name.as_ref().map(ToString::to_string),
            status: principal.status.as_str().to_owned(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreatedServicePrincipalResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) display_name: Option<String>,
    pub(crate) status: String,
    /// Raw API key, returned exactly once and never stored in the clear.
    pub(crate) api_key: String,
}

impl From<CreatedServicePrincipal> for CreatedServicePrincipalResponse {
    fn from(created: CreatedServicePrincipal) -> Self {
        let principal = ServicePrincipalResponse::from(&created.principal);
        Self {
            id: principal.id,
            name: principal.name,
            display_name: principal.display_name,
            status: principal.status,
            api_key: created.api_key,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct RotatedServicePrincipalKeyResponse {
    pub(crate) id: String,
    /// New raw API key, returned exactly once and never stored in the clear.
    pub(crate) api_key: String,
}

impl From<RotatedServicePrincipalKey> for RotatedServicePrincipalKeyResponse {
    fn from(rotated: RotatedServicePrincipalKey) -> Self {
        Self {
            id: rotated.principal.id.to_string(),
            api_key: rotated.api_key,
        }
    }
}
