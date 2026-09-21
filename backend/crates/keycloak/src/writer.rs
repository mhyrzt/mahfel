use crate::KeycloakAdmin;
use mahfel_application::ports::{AttributeError, BoxFuture, UserAttributeWriter};

impl From<crate::error::KeycloakError> for AttributeError {
    fn from(error: crate::error::KeycloakError) -> Self {
        match error {
            crate::error::KeycloakError::Unavailable(message) => Self::Unavailable(message),
            // A rejected service account is a configuration fault, not an outage.
            crate::error::KeycloakError::Unauthorized(message)
            | crate::error::KeycloakError::Unexpected(message) => Self::Unexpected(message),
        }
    }
}

impl UserAttributeWriter for KeycloakAdmin {
    fn set_user_attribute<'a>(
        &'a self,
        user_id: &'a str,
        name: &'a str,
        value: &'a str,
    ) -> BoxFuture<'a, Result<(), AttributeError>> {
        Box::pin(async move {
            KeycloakAdmin::set_user_attribute(self, user_id, name, value)
                .await
                .map_err(AttributeError::from)
        })
    }
}
