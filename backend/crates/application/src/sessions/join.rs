use super::error::SessionOperationError;
use crate::admission::{AdmissionOutcome, EnqueueAdmission, enqueue_admission};
use crate::ports::{
    AdmissionStore, BlockStore, ParticipantStore, ParticipationStore, RoomStore, SessionStore,
    StoreError,
};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequest, Participant, ParticipantError,
    ParticipationInterval, RoomId, SessionId, SessionStatus,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct JoinSession {
    pub actor: ActorId,
    pub session_id: SessionId,
    pub origin: AdmissionOrigin,
}

/// What a join attempt resolved to.
///
/// Three outcomes, not two: authorization can succeed while admission is still
/// undecided. A waiting principal gets no participant, no interval, and no
/// participation capability, so "cannot publish until admitted" is structural
/// rather than a UI convention.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JoinOutcome {
    Joined {
        participant: Participant,
        room_id: RoomId,
    },
    Waiting(AdmissionRequest),
}

impl JoinOutcome {
    /// The participant, when the join was admitted immediately.
    #[must_use]
    pub fn participant(self) -> Option<Participant> {
        match self {
            Self::Joined { participant, .. } => Some(participant),
            Self::Waiting(_) => None,
        }
    }

    #[must_use]
    pub fn is_waiting(&self) -> bool {
        matches!(self, Self::Waiting(_))
    }
}

/// Join a live session, reusing an existing participant on reconnect.
///
/// The single join path: admission layers on top of the same guard, so it cannot
/// be bypassed by another entry point. Authorization runs first and is
/// unchanged; the effective admission policy then decides whether the caller
/// enters or waits.
///
/// Every admitted join opens a participation interval, including a reconnect:
/// the prior interval closed when that connection ended, so a reconnect is a new
/// connected stretch even though it reuses the durable participant identity.
#[allow(clippy::too_many_arguments)]
pub async fn join_session<P, Q, A, R, S, B>(
    participants: &P,
    participation: &Q,
    admissions: &A,
    sessions: &R,
    rooms: &S,
    blocks: &B,
    command: JoinSession,
) -> Result<JoinOutcome, SessionOperationError>
where
    P: ParticipantStore + ?Sized,
    Q: ParticipationStore + ?Sized,
    A: AdmissionStore + ?Sized,
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
    B: BlockStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if session.status != SessionStatus::Live {
        return Err(SessionOperationError::NotLive);
    }
    let room = super::require_joinable(rooms, session.room_id, command.actor).await?;

    // A block bars entry before admission is even considered, so a blocked
    // principal never queues and cannot occupy a lobby slot. The room check runs
    // first because a room block already covers every session in it; the session
    // check then covers blocks narrower than a room block. The two are
    // independent durable facts, so neither writes rows for the other.
    super::require_not_blocked(blocks, room.id, command.session_id, command.actor).await?;

    if session.effective_admission(&room).requires_approval() {
        let outcome = enqueue_admission(
            admissions,
            sessions,
            rooms,
            EnqueueAdmission {
                actor: command.actor,
                target: mahfel_domain::AdmissionTarget::Session(command.session_id),
                origin: command.origin,
            },
            now,
        )
        .await?;
        return Ok(match outcome {
            AdmissionOutcome::Waiting(request) => JoinOutcome::Waiting(request),
        });
    }

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
        return Ok(JoinOutcome::Joined {
            participant: existing,
            room_id: room.id,
        });
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
    Ok(JoinOutcome::Joined {
        participant,
        room_id: room.id,
    })
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
