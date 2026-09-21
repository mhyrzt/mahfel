use super::error::SessionOperationError;
use crate::ports::ParticipantStore;
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
pub async fn leave_session<P>(
    participants: &P,
    command: LeaveSession,
) -> Result<(), SessionOperationError>
where
    P: ParticipantStore + ?Sized,
{
    let mut participant = participants
        .get(command.participant_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if participant.actor_id != command.actor {
        return Err(SessionOperationError::NotFound);
    }
    participant.leave(OffsetDateTime::now_utc())?;
    participants.save(&participant).await?;
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
