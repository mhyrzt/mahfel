use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, IntervalId, ParticipantId, ParticipationInterval, SessionId};

/// A participant's connected time inside one session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticipantDuration {
    pub participant_id: ParticipantId,
    pub actor_id: ActorId,
    /// Total non-superseded seconds. A session-scoped total.
    pub connected_seconds: i64,
    /// Intervals still open for this participant at query time.
    pub open_intervals: u32,
}

/// Persistence for durable participation records (M4-03).
///
/// Intervals are append-only: a correction supersedes the original row rather
/// than updating it, so history survives. Methods return [`BoxFuture`] so the
/// port is dyn-compatible (ADR 0023).
pub trait ParticipationStore: Send + Sync {
    /// Open an interval. Returns [`StoreError::Conflict`] when the participant
    /// already has one open, which the partial unique index also enforces.
    fn open_interval<'a>(
        &'a self,
        interval: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Close the participant's open interval if one exists.
    ///
    /// Idempotent: a duplicate disconnect finds no open interval and returns
    /// `Ok(None)` rather than moving an end time. Returns the closed interval's
    /// id when there was one.
    fn close_open_interval(
        &self,
        participant_id: ParticipantId,
        ended_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<Option<IntervalId>, StoreError>>;

    /// Close every open interval in a session.
    ///
    /// The end-of-session sweep. Idempotent, and returns how many it closed so
    /// the caller can audit a real change rather than a repeat.
    fn close_open_intervals_for_session(
        &self,
        session_id: SessionId,
        ended_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>>;

    fn get(
        &self,
        id: IntervalId,
    ) -> BoxFuture<'_, Result<Option<ParticipationInterval>, StoreError>>;

    /// Every interval for one participant in one session, oldest first.
    ///
    /// Includes superseded rows so callers can show the correction history.
    fn list_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>>;

    /// Every interval recorded in one session, oldest first.
    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>>;

    /// Persist a correction: the original is marked superseded and the
    /// replacement is inserted in one transaction, so a reader never sees a
    /// participant with zero active intervals because the swap was partial.
    fn supersede_interval<'a>(
        &'a self,
        original: &'a ParticipationInterval,
        replacement: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>>;
}
