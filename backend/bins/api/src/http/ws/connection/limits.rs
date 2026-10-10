use std::time::Instant;

pub(super) struct FrameBudget {
    tokens: f64,
    updated: Instant,
}

impl FrameBudget {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            tokens: 100.0,
            updated: now,
        }
    }

    pub(super) fn allow(&mut self, now: Instant) -> bool {
        self.tokens =
            (self.tokens + now.duration_since(self.updated).as_secs_f64() * 100.0).min(100.0);
        self.updated = now;
        if self.tokens < 1.0 {
            return false;
        }
        self.tokens -= 1.0;
        true
    }
}

#[derive(Default)]
pub(super) struct RenewalBudget {
    missed: u8,
}

impl RenewalBudget {
    pub(super) fn succeeded(&mut self) {
        self.missed = 0;
    }
    pub(super) fn failed(&mut self) -> bool {
        self.missed = self.missed.saturating_add(1);
        self.missed < 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn inbound_burst_and_refill_are_bounded() {
        let now = Instant::now();
        let mut budget = FrameBudget::new(now);
        for _ in 0..100 {
            assert!(budget.allow(now));
        }
        assert!(!budget.allow(now));
        assert!(budget.allow(now + Duration::from_millis(10)));
        assert!(!budget.allow(now + Duration::from_millis(10)));
    }

    #[test]
    fn transient_presence_failures_recover_but_stop_before_ttl() {
        let mut budget = RenewalBudget::default();
        assert!(budget.failed());
        assert!(budget.failed());
        budget.succeeded();
        assert!(budget.failed());
        assert!(budget.failed());
        assert!(!budget.failed());
        assert!(Duration::from_secs(30) < crate::http::ws::state::SESSION_PRESENCE_TTL);
    }
}
