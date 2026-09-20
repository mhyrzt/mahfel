use crate::ids::entity_id;
use crate::status::status_conversion;
use crate::validation::DisplayName;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

// Identifier shared by every actor in one PostgreSQL id space.
//
// Users and service principals both own an `ActorId`, so a room has one owner
// column and one membership column regardless of who (a person or a machine)
// owns it.
entity_id!(ActorId);

/// What kind of actor an id refers to.
///
/// The kind is carried alongside the actor id in an authenticated principal so
/// authorization can reason about a service principal without a database
/// lookup, and is stored on `mahfel.actors` as the discriminator.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Service,
}

/// Lifecycle of a service principal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServicePrincipalStatus {
    Active,
    Revoked,
}

/// Why a service principal state transition was rejected.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ServicePrincipalError {
    #[error("service principal is already revoked")]
    AlreadyRevoked,
}

/// A first-class machine actor.
///
/// A service principal is not a user row and is never impersonated as one: it
/// has its own `ActorId`, its own credentials, and can own rooms and hold
/// memberships exactly like a user.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServicePrincipal {
    pub id: ActorId,
    /// Stable, unique operator-facing name (for example `lms-core`).
    pub name: DisplayName,
    /// Optional human-readable label.
    pub display_name: Option<DisplayName>,
    pub status: ServicePrincipalStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl ServicePrincipal {
    #[must_use]
    pub fn create(
        name: DisplayName,
        display_name: Option<DisplayName>,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: ActorId::new(),
            name,
            display_name,
            status: ServicePrincipalStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Revoke the principal. Revocation is terminal; a revoked principal can no
    /// longer authenticate.
    pub fn revoke(&mut self, now: OffsetDateTime) -> Result<(), ServicePrincipalError> {
        if self.status == ServicePrincipalStatus::Revoked {
            return Err(ServicePrincipalError::AlreadyRevoked);
        }
        self.status = ServicePrincipalStatus::Revoked;
        self.updated_at = now;
        Ok(())
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == ServicePrincipalStatus::Active
    }
}

status_conversion!(ActorKind, "actor", { User => "user", Service => "service" });
status_conversion!(ServicePrincipalStatus, "service principal", {
    Active => "active",
    Revoked => "revoked",
});

#[cfg(test)]
mod tests {
    use super::*;

    fn principal() -> ServicePrincipal {
        ServicePrincipal::create(
            DisplayName::new("lms-core".to_owned()).expect("valid name"),
            Some(DisplayName::new("LMS Core".to_owned()).expect("valid display name")),
            OffsetDateTime::now_utc(),
        )
    }

    #[test]
    fn actor_kind_storage_names_are_stable() {
        assert_eq!(ActorKind::User.as_str(), "user");
        assert_eq!(ActorKind::Service.as_str(), "service");
        assert_eq!(
            ActorKind::try_from("service").expect("known kind"),
            ActorKind::Service
        );
        assert!(ActorKind::try_from("system").is_err());
    }

    #[test]
    fn service_principal_status_storage_names_are_stable() {
        assert_eq!(ServicePrincipalStatus::Active.as_str(), "active");
        assert_eq!(ServicePrincipalStatus::Revoked.as_str(), "revoked");
        assert!(ServicePrincipalStatus::try_from("disabled").is_err());
    }

    #[test]
    fn revoke_is_state_checked_and_terminal() {
        let now = OffsetDateTime::now_utc();
        let mut principal = principal();
        assert!(principal.is_active());
        assert!(principal.revoke(now).is_ok());
        assert!(!principal.is_active());
        assert_eq!(
            principal.revoke(now).expect_err("already revoked"),
            ServicePrincipalError::AlreadyRevoked
        );
    }
}
