use super::error::SessionOperationError;
use crate::ports::{ParticipantStore, ParticipationStore};
use mahfel_domain::{ActorId, ParticipantId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct LeaveSession {
    pub actor: ActorId,
    pub participant_id: ParticipantId,
}

/// End one actor's participation.
///
/// A missing participant and a participant owned by another actor collapse to
/// `NotFound` so a caller cannot probe which participants exist.
///
/// Leaving closes the participant's open interval in the same call, so
/// connected time is recorded even when the transport goes away without saying
/// goodbye. Closing is idempotent; a second disconnect finds no open interval.
pub async fn leave_session<P, Q>(
    participants: &P,
    participation: &Q,
    command: LeaveSession,
) -> Result<(), SessionOperationError>
where
    P: ParticipantStore + ?Sized,
    Q: ParticipationStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    let mut participant = participants
        .get(command.participant_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if participant.actor_id != command.actor {
        return Err(SessionOperationError::NotFound);
    }
    participant.leave(now)?;
    participants.save(&participant).await?;
    super::join::close_interval(participation, participant.id, now).await?;
    tracing::info!(
        target: "audit",
        action = "participant.leave",
        actor_id = %command.actor,
        session_id = %participant.session_id,
        participant_id = %participant.id,
        outcome = "ok"
    );
    Ok(())
}
