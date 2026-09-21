use super::error::SessionOperationError;
use crate::ports::{ParticipantStore, ParticipationStore, RoomStore, SessionStore, StoreError};
use mahfel_domain::{
    ActorId, Participant, ParticipantError, ParticipationInterval, SessionId, SessionStatus,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct JoinSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Join a live session, reusing an existing participant on reconnect.
///
/// The single join path: M4-06 layers admission policy on top of the same
/// guard, so it must not be bypassed by another entry point.
///
/// Every accepted join opens a participation interval, including a reconnect:
/// the prior interval closed when that connection ended, so a reconnect is a
/// new connected stretch even though it reuses the durable participant
/// identity. A fresh join and a reconnect therefore differ only in whether a
/// participant row is created.
pub async fn join_session<P, Q, R, S>(
    participants: &P,
    participation: &Q,
    sessions: &R,
    rooms: &S,
    command: JoinSession,
) -> Result<Participant, SessionOperationError>
where
    P: ParticipantStore + ?Sized,
    Q: ParticipationStore + ?Sized,
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if session.status != SessionStatus::Live {
        return Err(SessionOperationError::NotLive);
    }
    super::require_joinable(rooms, session.room_id, command.actor).await?;

    if let Some(existing) = participants
        .get_active(command.session_id, command.actor)
        .await?
    {
        open_interval(participation, &existing, now).await?;
        tracing::info!(
            target: "audit",
            action = "participant.reconnect",
            actor_id = %command.actor,
            session_id = %command.session_id,
            participant_id = %existing.id,
            outcome = "ok"
        );
        return Ok(existing);
    }

    let participant = Participant::join(command.session_id, command.actor, now);
    participants
        .insert(&participant)
        .await
        .map_err(|error| match error {
            StoreError::Conflict(_) => {
                SessionOperationError::Participant(ParticipantError::AlreadyActive)
            }
            other => SessionOperationError::Store(other),
        })?;
    open_interval(participation, &participant, now).await?;
    tracing::info!(
        target: "audit",
        action = "participant.join",
        actor_id = %command.actor,
        session_id = %command.session_id,
        participant_id = %participant.id,
        outcome = "ok"
    );
    Ok(participant)
}

/// Open the interval for an accepted join.
///
/// A conflict means the participant already has an open interval, which a
/// reconnect after a missed disconnect can produce. That is not a join failure:
/// the participant is present either way, and the open interval already records
/// it, so the conflict is tolerated.
async fn open_interval<Q>(
    participation: &Q,
    participant: &Participant,
    now: OffsetDateTime,
) -> Result<(), SessionOperationError>
where
    Q: ParticipationStore + ?Sized,
{
    let interval = ParticipationInterval::open(
        participant.id,
        participant.session_id,
        participant.actor_id,
        now,
    );
    match participation.open_interval(&interval).await {
        Ok(()) => Ok(()),
        Err(StoreError::Conflict(_)) => {
            tracing::debug!(
                participant_id = %participant.id,
                "an open interval already covers this join"
            );
            Ok(())
        }
        Err(other) => Err(SessionOperationError::Store(other)),
    }
}

/// Close a participant's open interval, if one exists.
pub(crate) async fn close_interval<Q>(
    participation: &Q,
    participant_id: mahfel_domain::ParticipantId,
    now: OffsetDateTime,
) -> Result<(), SessionOperationError>
where
    Q: ParticipationStore + ?Sized,
{
    participation
        .close_open_interval(participant_id, now)
        .await?;
    Ok(())
}
