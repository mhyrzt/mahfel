use super::error::ParticipationError;
use crate::ports::ParticipationStore;
use mahfel_domain::{ActorId, IntervalId, ParticipantId, ParticipationInterval, SessionId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct OpenInterval {
    pub participant_id: ParticipantId,
    pub session_id: SessionId,
    pub actor_id: ActorId,
}

/// Open a participation interval for a participant.
///
/// Called from the join path. The partial unique index makes a duplicated open
/// a conflict, so a racing join cannot record two open stretches.
pub async fn open_interval<P>(
    participation: &P,
    command: OpenInterval,
    now: OffsetDateTime,
) -> Result<ParticipationInterval, ParticipationError>
where
    P: ParticipationStore + ?Sized,
{
    let interval = ParticipationInterval::open(
        command.participant_id,
        command.session_id,
        command.actor_id,
        now,
    );
    participation
        .open_interval(&interval)
        .await
        .map_err(|error| match error {
            crate::ports::StoreError::Conflict(_) => {
                ParticipationError::Conflict(mahfel_domain::IntervalError::AlreadyOpen)
            }
            other => ParticipationError::Store(other),
        })?;
    Ok(interval)
}

#[derive(Debug, Clone)]
pub struct CloseInterval {
    pub participant_id: ParticipantId,
}

/// Close the participant's open interval, if any.
///
/// Idempotent by design: a duplicate disconnect, or a disconnect for a
/// participant that never opened one, returns `None` instead of moving an end
/// time. That is what makes the no-grace policy safe under retries.
pub async fn close_interval<P>(
    participation: &P,
    command: CloseInterval,
    now: OffsetDateTime,
) -> Result<Option<IntervalId>, ParticipationError>
where
    P: ParticipationStore + ?Sized,
{
    Ok(participation
        .close_open_interval(command.participant_id, now)
        .await?)
}

/// Close every open interval in a session, stopping the clock at `ended_at`.
///
/// The end-of-session sweep (AC 5). Called when a session ends, so no interval
/// survives it. Idempotent, and returns how many it closed so the caller can
/// audit a real change rather than a repeat.
pub async fn close_session_intervals<P>(
    participation: &P,
    session_id: SessionId,
    ended_at: OffsetDateTime,
) -> Result<u64, ParticipationError>
where
    P: ParticipationStore + ?Sized,
{
    Ok(participation
        .close_open_intervals_for_session(session_id, ended_at)
        .await?)
}
