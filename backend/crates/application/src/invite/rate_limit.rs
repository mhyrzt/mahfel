//! Bounded rate limiting for invite-code attempts (M4-13).
//!
//! A shared code is a bearer capability, so redemption must not help enumerate
//! valid codes (ADR 0021). Counting lives behind [`CoordinationStore`] because
//! the window must be shared across replicas and is replaceable state. Both a
//! principal and a source address are counted: either alone is weak.

use crate::ports::{CoordinationError, CoordinationStore};
use sha2::{Digest, Sha256};
use std::time::Duration;

pub const INVITE_ATTEMPT_FAMILY: &str = "invite-attempt";

pub const MAX_ATTEMPTS_PER_PRINCIPAL: u64 = 10;
/// Higher than the per-principal budget because one address may legitimately
/// hold several participants, for example behind an office NAT.
pub const MAX_ATTEMPTS_PER_ADDRESS: u64 = 30;
pub const ATTEMPT_WINDOW: Duration = Duration::from_secs(60);

/// A closed enum rather than a free string, so a key family cannot be invented
/// at a call site and the two budgets cannot drift apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateLimitScope {
    Principal,
    Address,
}

impl RateLimitScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Principal => "principal",
            Self::Address => "address",
        }
    }

    #[must_use]
    pub const fn limit(self) -> u64 {
        match self {
            Self::Principal => MAX_ATTEMPTS_PER_PRINCIPAL,
            Self::Address => MAX_ATTEMPTS_PER_ADDRESS,
        }
    }
}

/// Whether an attempt may proceed, and what to do when it may not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptDecision {
    Allowed {
        remaining: u64,
    },
    /// Must be rejected with the same error as an unknown code, so a limited
    /// attempt is indistinguishable from an invalid one.
    Limited {
        retry_after: Duration,
    },
}

impl AttemptDecision {
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        matches!(self, Self::Allowed { .. })
    }
}

/// Policy lives here and storage behind [`CoordinationStore`], so the budgets
/// are testable without Redis.
#[derive(Clone, Copy, Debug)]
pub struct AttemptLimiter {
    window: Duration,
}

impl Default for AttemptLimiter {
    fn default() -> Self {
        Self {
            window: ATTEMPT_WINDOW,
        }
    }
}

impl AttemptLimiter {
    /// Counters expire `window` after the first attempt in them, so the window
    /// is fixed rather than sliding: cheaper, and the goal is bounding volume
    /// rather than precise smoothing.
    #[must_use]
    pub const fn with_window(window: Duration) -> Self {
        Self { window }
    }

    /// Count one attempt and report whether it is within budget.
    ///
    /// Both scopes are counted before either is judged, so a rejected attempt
    /// still consumes the other scope's budget and a limited caller cannot probe
    /// for free.
    ///
    /// # Errors
    /// [`CoordinationError`] when the counter cannot be read. A limiter that
    /// cannot count must not silently allow.
    pub async fn check<C>(
        &self,
        coordination: &C,
        principal: &crate::ports::Principal,
        source_address: Option<&str>,
    ) -> Result<AttemptDecision, CoordinationError>
    where
        C: CoordinationStore + ?Sized,
    {
        let principal_key = attempt_key(
            RateLimitScope::Principal,
            &digest(principal.actor_id.as_uuid().as_bytes()),
        );
        let principal_count = coordination
            .increment_with_ttl(&principal_key, self.window)
            .await?;

        // An address is not always available; the principal budget alone is the
        // stricter of the two anyway.
        let address_count = match source_address {
            Some(address) => {
                let key = attempt_key(RateLimitScope::Address, &digest(address.as_bytes()));
                Some(coordination.increment_with_ttl(&key, self.window).await?)
            }
            None => None,
        };

        let principal_limit = RateLimitScope::Principal.limit();
        let over_principal = principal_count > principal_limit;
        let over_address = address_count
            .map(|count| count > RateLimitScope::Address.limit())
            .unwrap_or(false);

        if over_principal || over_address {
            return Ok(AttemptDecision::Limited {
                retry_after: self.window,
            });
        }

        let remaining = principal_limit.saturating_sub(principal_count);
        Ok(AttemptDecision::Allowed { remaining })
    }
}

/// The raw identifier never reaches a key. Not a secrecy measure — an address
/// is guessable — but namespace hygiene: the key space stays opaque.
#[must_use]
pub fn digest(value: &[u8]) -> String {
    let hex = Sha256::digest(value);
    let mut out = String::with_capacity(hex.len() * 2);
    for byte in hex {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// The adapter owns the configured prefix; this composes the family and scope
/// under it, next to the policy that depends on the key shape.
fn attempt_key(scope: RateLimitScope, digest: &str) -> String {
    format!("{INVITE_ATTEMPT_FAMILY}:{}:{digest}", scope.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{BoxFuture, Principal};
    use mahfel_domain::{ActorId, ActorKind};
    use std::sync::{Arc, Mutex};

    /// In-memory counter double, storing a count per key and never expiring:
    /// the worst case the limiter must still bound.
    #[derive(Clone, Default)]
    struct CountingCoordination {
        inner: Arc<Mutex<Vec<(String, u64)>>>,
    }

    impl CountingCoordination {
        fn count(&self, key: &str) -> u64 {
            self.inner
                .lock()
                .expect("lock")
                .iter()
                .find(|(candidate, _)| candidate == key)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        }
    }

    impl CoordinationStore for CountingCoordination {
        fn set_with_ttl<'a>(
            &'a self,
            _key: &'a str,
            _value: &'a [u8],
            _ttl: Duration,
        ) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async move { Ok(()) })
        }

        fn get<'a>(
            &'a self,
            _key: &'a str,
        ) -> BoxFuture<'a, Result<Option<Vec<u8>>, CoordinationError>> {
            Box::pin(async move { Ok(None) })
        }

        fn delete<'a>(&'a self, _key: &'a str) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async move { Ok(()) })
        }

        fn publish<'a>(
            &'a self,
            _channel: &'a str,
            _payload: &'a [u8],
        ) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async move { Ok(()) })
        }

        fn increment_with_ttl<'a>(
            &'a self,
            key: &'a str,
            _ttl: Duration,
        ) -> BoxFuture<'a, Result<u64, CoordinationError>> {
            Box::pin(async move {
                let mut inner = self.inner.lock().expect("lock");
                match inner.iter_mut().find(|(candidate, _)| candidate == key) {
                    Some((_, count)) => {
                        *count += 1;
                        Ok(*count)
                    }
                    None => {
                        inner.push((key.to_owned(), 1));
                        Ok(1)
                    }
                }
            })
        }
    }

    fn principal() -> Principal {
        Principal {
            actor_id: ActorId::new(),
            kind: ActorKind::User,
            display_name: None,
            avatar_url: None,
            email: None,
            email_verified: false,
            username: None,
            roles: Vec::new(),
        }
    }

    #[tokio::test]
    async fn attempts_are_allowed_within_budget() {
        let coordination = CountingCoordination::default();
        let limiter = AttemptLimiter::default();
        for attempt in 1..=MAX_ATTEMPTS_PER_PRINCIPAL {
            let decision = limiter
                .check(&coordination, &principal(), Some("198.51.100.7"))
                .await
                .expect("counted");
            assert!(
                decision.is_allowed(),
                "attempt {attempt} is within the principal budget"
            );
        }
    }

    #[tokio::test]
    async fn the_principal_budget_is_enforced() {
        let coordination = CountingCoordination::default();
        let limiter = AttemptLimiter::default();
        let principal = principal();
        for _ in 0..MAX_ATTEMPTS_PER_PRINCIPAL {
            assert!(
                limiter
                    .check(&coordination, &principal, None)
                    .await
                    .expect("counted")
                    .is_allowed()
            );
        }
        let decision = limiter
            .check(&coordination, &principal, None)
            .await
            .expect("counted");
        assert_eq!(
            decision,
            AttemptDecision::Limited {
                retry_after: ATTEMPT_WINDOW
            }
        );
    }

    #[tokio::test]
    async fn the_address_budget_is_enforced_independently_of_the_principal() {
        let coordination = CountingCoordination::default();
        let limiter = AttemptLimiter::default();
        // One address, a fresh principal each time: the address budget must
        // still stop the spray.
        for _ in 0..MAX_ATTEMPTS_PER_ADDRESS {
            assert!(
                limiter
                    .check(&coordination, &principal(), Some("203.0.113.9"))
                    .await
                    .expect("counted")
                    .is_allowed()
            );
        }
        let decision = limiter
            .check(&coordination, &principal(), Some("203.0.113.9"))
            .await
            .expect("counted");
        assert!(!decision.is_allowed());
    }

    #[tokio::test]
    async fn scopes_do_not_share_a_budget() {
        let coordination = CountingCoordination::default();
        let limiter = AttemptLimiter::default();
        let principal = principal();
        // Drain the principal budget against one address.
        for _ in 0..MAX_ATTEMPTS_PER_PRINCIPAL {
            limiter
                .check(&coordination, &principal, Some("203.0.113.9"))
                .await
                .expect("counted");
        }
        // A different principal from a different address still has budget.
        let other = crate::invite::rate_limit::tests::principal();
        let decision = limiter
            .check(&coordination, &other, Some("198.51.100.7"))
            .await
            .expect("counted");
        assert!(decision.is_allowed());
    }

    #[tokio::test]
    async fn a_rejected_attempt_still_consumes_budget() {
        let coordination = CountingCoordination::default();
        let limiter = AttemptLimiter::default();
        let principal = principal();
        for _ in 0..(MAX_ATTEMPTS_PER_PRINCIPAL + 5) {
            limiter
                .check(&coordination, &principal, None)
                .await
                .expect("counted");
        }
        let any_key: String = coordination
            .inner
            .lock()
            .expect("lock")
            .first()
            .map(|(key, _)| key.clone())
            .expect("a counter exists");
        assert!(
            coordination.count(&any_key) > MAX_ATTEMPTS_PER_PRINCIPAL,
            "limited attempts must keep counting, not reset the window"
        );
    }

    #[test]
    fn keys_never_contain_the_raw_identifier() {
        let address = "203.0.113.9";
        let key = attempt_key(RateLimitScope::Address, &digest(address.as_bytes()));
        assert!(
            !key.contains(address),
            "the raw address must not be in a key"
        );
        assert!(key.contains("invite-attempt:address:"));
        assert_eq!(
            key,
            attempt_key(RateLimitScope::Address, &digest(address.as_bytes())),
            "the digest is deterministic"
        );
    }

    #[test]
    fn digests_are_fixed_length_hex() {
        for value in ["", "a", "203.0.113.9"] {
            let digest = digest(value.as_bytes());
            assert_eq!(digest.len(), 64);
            assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn scope_budgets_are_documented() {
        assert!(RateLimitScope::Address.limit() > RateLimitScope::Principal.limit());
        assert_eq!(RateLimitScope::Principal.as_str(), "principal");
        assert_eq!(RateLimitScope::Address.as_str(), "address");
    }
}
