use super::error::ParticipationError;
use crate::ports::{ParticipationStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, CorrectionReason, IntervalId, ParticipationInterval};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CorrectInterval {
    pub actor: ActorId,
    /// The interval being corrected. It is superseded, never mutated.
    pub interval_id: IntervalId,
    pub reason: CorrectionReason,
    pub started_at: OffsetDateTime,
    /// `None` reopens the interval, which is only valid for a correction that
    /// says the participant is in fact still connected.
    pub ended_at: Option<OffsetDateTime>,
}

/// Manager-only: replace an interval with a corrected one.
///
/// The original row is marked superseded and retained, so the original
/// measurement and the reason it changed both survive. The replacement
/// is inserted in the same transaction, so a reader never observes the
/// participant with zero active intervals because the swap was partial.
pub async fn correct_interval<P, S, R>(
    participation: &P,
    sessions: &S,
    rooms: &R,
    command: CorrectInterval,
) -> Result<ParticipationInterval, ParticipationError>
where
    P: ParticipationStore + ?Sized,
    S: SessionStore + ?Sized,
    R: crate::ports::RoomStore + ?Sized,
{
    if let Some(ended_at) = command.ended_at
        && ended_at < command.started_at
    {
        return Err(ParticipationError::InvalidCorrection(
            "end must not precede start",
        ));
    }

    let original = participation
        .get(command.interval_id)
        .await?
        .ok_or(ParticipationError::NotFound)?;
    if original.is_superseded() {
        return Err(ParticipationError::Conflict(
            mahfel_domain::IntervalError::AlreadyClosed,
        ));
    }

    let session = sessions
        .get(original.session_id)
        .await?
        .ok_or(ParticipationError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;

    // A still-open interval cannot be corrected to closed after the session
    // ended, or the record would outlive the session it belongs to.
    if let (Some(ended_at), Some(session_end)) = (command.ended_at, session.ended_at)
        && ended_at > session_end
    {
        return Err(ParticipationError::InvalidCorrection(
            "an interval cannot end after its session",
        ));
    }

    let mut replacement = ParticipationInterval::open(
        original.participant_id,
        original.session_id,
        original.actor_id,
        command.started_at,
    );
    if let Some(ended_at) = command.ended_at {
        replacement.close(ended_at)?;
    }
    let mut superseded = original.clone();
    superseded.supersede(replacement.id, command.reason, command.started_at)?;

    participation
        .supersede_interval(&superseded, &replacement)
        .await
        .map_err(|error| match error {
            crate::ports::StoreError::Conflict(_) => {
                ParticipationError::Conflict(mahfel_domain::IntervalError::AlreadyClosed)
            }
            other => ParticipationError::Store(other),
        })?;

    tracing::info!(
        target: "audit",
        action = "participation.correct",
        actor_id = %command.actor,
        session_id = %original.session_id,
        participant_id = %original.participant_id,
        original_interval_id = %original.id,
        replacement_interval_id = %replacement.id,
        reason = command.reason.as_str(),
        outcome = "ok"
    );

    Ok(replacement)
}
