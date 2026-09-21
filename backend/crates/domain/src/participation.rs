//! Durable presence records for one participant in one session.
//!
//! A [`ParticipationInterval`] is the durable fact behind "who was connected,
//! and for how long". It is appended by the join path, closed by the leave
//! path, and never derived from ephemeral presence: losing Redis or restarting
//! the API must not change what is already recorded.
//!
//! Intervals are append-only. A correction does not overwrite the original; it
//! supersedes it, so the original measurement and the reason it was changed
//! both survive.

use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::participant::ParticipantId;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(IntervalId);

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum IntervalError {
    #[error("interval is already closed")]
    AlreadyClosed,
    #[error("interval end must not precede its start")]
    NegativeDuration,
    #[error("an open interval already exists for this participant")]
    AlreadyOpen,
}

/// Why an interval was superseded by a correction.
///
/// A closed set so the reason is auditable and queryable rather than free text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionReason {
    /// The recorded end was wrong (for example a missed disconnect).
    WrongEnd,
    /// The recorded start was wrong.
    WrongStart,
    /// The whole interval should not have been recorded.
    Erroneous,
    /// An operator applied a manual adjustment.
    ManualAdjustment,
}

crate::status::status_conversion!(CorrectionReason, "correction reason", {
    WrongEnd => "wrong_end",
    WrongStart => "wrong_start",
    Erroneous => "erroneous",
    ManualAdjustment => "manual_adjustment",
});

/// One connected stretch for one participant.
///
/// `ended_at` is `None` while the participant is connected. At most one
/// interval per participant is open at a time, which the partial unique index
/// enforces in the database.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParticipationInterval {
    pub id: IntervalId,
    pub participant_id: ParticipantId,
    pub session_id: SessionId,
    pub actor_id: ActorId,
    pub started_at: OffsetDateTime,
    pub ended_at: Option<OffsetDateTime>,
    /// Set when a correction superseded this interval. The row is retained.
    pub superseded_by: Option<IntervalId>,
    pub correction_reason: Option<CorrectionReason>,
    pub created_at: OffsetDateTime,
}

impl ParticipationInterval {
    /// Open an interval at `started_at`.
    ///
    /// Callers must have established there is no open interval for the
    /// participant; the partial unique index turns a racing open into a
    /// conflict rather than a second open row.
    #[must_use]
    pub fn open(
        participant_id: ParticipantId,
        session_id: SessionId,
        actor_id: ActorId,
        started_at: OffsetDateTime,
    ) -> Self {
        Self {
            id: IntervalId::new(),
            participant_id,
            session_id,
            actor_id,
            started_at,
            ended_at: None,
            superseded_by: None,
            correction_reason: None,
            created_at: started_at,
        }
    }

    /// Close an open interval. Closing a closed one is a conflict, so a
    /// duplicated disconnect cannot move an end time.
    pub fn close(&mut self, ended_at: OffsetDateTime) -> Result<(), IntervalError> {
        if self.ended_at.is_some() {
            return Err(IntervalError::AlreadyClosed);
        }
        if ended_at < self.started_at {
            return Err(IntervalError::NegativeDuration);
        }
        self.ended_at = Some(ended_at);
        Ok(())
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.ended_at.is_none()
    }

    #[must_use]
    pub fn is_superseded(&self) -> bool {
        self.superseded_by.is_some()
    }

    /// Seconds this interval contributed, or `None` while it is open.
    ///
    /// Saturates at zero so a corrected interval cannot subtract from a total.
    #[must_use]
    pub fn duration_seconds(&self, now: OffsetDateTime) -> i64 {
        let end = self.ended_at.unwrap_or(now);
        (end - self.started_at).whole_seconds().max(0)
    }

    /// Mark this interval as replaced by `replacement`, with a reason.
    ///
    /// The row is not deleted or mutated beyond this marker: the original
    /// measurement stays readable, and the marker names what replaced it.
    ///
    /// Superseding an open interval also closes it, because the replacement
    /// takes over as the live record. Leaving it open would let a superseded
    /// interval satisfy the one-open-interval invariant and double-count.
    pub fn supersede(
        &mut self,
        replacement: IntervalId,
        reason: CorrectionReason,
        now: OffsetDateTime,
    ) -> Result<(), IntervalError> {
        if self.is_superseded() {
            return Err(IntervalError::AlreadyClosed);
        }
        if self.ended_at.is_none() {
            self.ended_at = Some(now);
        }
        self.superseded_by = Some(replacement);
        self.correction_reason = Some(reason);
        Ok(())
    }
}

/// Whether two intervals belonging to one participant overlap.
///
/// Half-open comparison: an interval ending exactly when the next begins is
/// adjacent, not overlapping, so a disconnect and immediate reconnect are
/// valid. An open interval extends to `now`.
#[must_use]
pub fn intervals_overlap(
    a_start: OffsetDateTime,
    a_end: Option<OffsetDateTime>,
    b_start: OffsetDateTime,
    b_end: Option<OffsetDateTime>,
    now: OffsetDateTime,
) -> bool {
    let a_end = a_end.unwrap_or(now);
    let b_end = b_end.unwrap_or(now);
    a_start < b_end && b_start < a_end
}

/// Total connected seconds for a participant's intervals.
///
/// Superseded intervals are excluded so a correction changes the total once,
/// not twice. Open intervals count to `now` unless `closes_at` is supplied,
/// which the end-of-session sweep uses to stop the clock at session end.
#[must_use]
pub fn total_connected_seconds(
    intervals: &[ParticipationInterval],
    now: OffsetDateTime,
    closes_at: Option<OffsetDateTime>,
) -> i64 {
    intervals
        .iter()
        .filter(|interval| !interval.is_superseded())
        .map(|interval| {
            let end = interval.ended_at.or(closes_at);
            let end = end.unwrap_or(now);
            (end - interval.started_at).whole_seconds().max(0)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(seconds)
    }

    fn interval(start: i64, end: Option<i64>) -> ParticipationInterval {
        let mut value = ParticipationInterval::open(
            ParticipantId::new(),
            SessionId::new(),
            ActorId::new(),
            at(start),
        );
        if let Some(end) = end {
            value.close(at(end)).expect("close");
        }
        value
    }

    #[test]
    fn open_interval_has_no_end() {
        let value = interval(0, None);
        assert!(value.is_open());
        assert!(value.ended_at.is_none());
        assert!(!value.is_superseded());
    }

    #[test]
    fn closing_is_exclusive_with_the_start() {
        let mut value = interval(10, None);
        assert_eq!(
            value.close(at(9)).expect_err("negative"),
            IntervalError::NegativeDuration
        );
        assert_eq!(value.close(at(10)).expect("zero-length is valid"), ());
        assert!(!value.is_open());
        assert_eq!(
            value.close(at(20)).expect_err("already closed"),
            IntervalError::AlreadyClosed
        );
        assert_eq!(
            value.ended_at,
            Some(at(10)),
            "a rejected close must not move the end"
        );
    }

    #[test]
    fn adjacent_intervals_do_not_overlap() {
        assert!(!intervals_overlap(
            at(0),
            Some(at(10)),
            at(10),
            Some(at(20)),
            at(30)
        ));
        assert!(intervals_overlap(
            at(0),
            Some(at(11)),
            at(10),
            Some(at(20)),
            at(30)
        ));
    }

    #[test]
    fn open_intervals_extend_to_now_for_overlap() {
        assert!(intervals_overlap(at(0), None, at(5), Some(at(10)), at(30)));
        assert!(!intervals_overlap(at(0), None, at(10), Some(at(20)), at(5)));
    }

    #[test]
    fn superseding_preserves_the_original() {
        let mut original = interval(0, Some(10));
        let replacement = IntervalId::new();
        original
            .supersede(replacement, CorrectionReason::WrongEnd, at(20))
            .expect("supersede");
        assert_eq!(original.superseded_by, Some(replacement));
        assert_eq!(original.correction_reason, Some(CorrectionReason::WrongEnd));
        assert_eq!(original.started_at, at(0), "the original measurement stays");
        assert_eq!(
            original
                .supersede(replacement, CorrectionReason::WrongStart, at(20))
                .expect_err("already superseded"),
            IntervalError::AlreadyClosed
        );
    }

    #[test]
    fn superseding_an_open_interval_closes_it() {
        let mut open = interval(0, None);
        assert!(open.is_open());
        open.supersede(IntervalId::new(), CorrectionReason::WrongEnd, at(30))
            .expect("supersede");
        assert!(
            !open.is_open(),
            "a superseded interval must not stay open and satisfy the \
             one-open-interval invariant"
        );
        assert_eq!(open.ended_at, Some(at(30)));
    }

    #[test]
    fn total_excludes_superseded_and_clamps_open_intervals() {
        let mut first = interval(0, Some(60));
        let second = interval(60, Some(120));
        let third = interval(120, None);

        assert_eq!(
            total_connected_seconds(
                &[first.clone(), second.clone(), third.clone()],
                at(200),
                None
            ),
            200
        );
        // The end-of-session sweep stops the open interval at session end.
        assert_eq!(
            total_connected_seconds(
                &[first.clone(), second.clone(), third.clone()],
                at(10_000),
                Some(at(180))
            ),
            180
        );
        // A superseded interval must not also count.
        first
            .supersede(IntervalId::new(), CorrectionReason::Erroneous, at(60))
            .expect("supersede");
        assert_eq!(
            total_connected_seconds(&[first, second, third], at(200), None),
            140
        );
    }

    #[test]
    fn duration_saturates_at_zero_for_a_backwards_correction() {
        let mut value = interval(100, None);
        // Force an end before the start to prove the clamp, bypassing `close`.
        value.ended_at = Some(at(50));
        assert_eq!(value.duration_seconds(at(0)), 0);
    }
}
