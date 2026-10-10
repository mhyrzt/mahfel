use super::error::ParticipationError;
use crate::ports::{ParticipationStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{
    ActorId, ParticipantId, ParticipationInterval, SessionId, total_connected_seconds,
};
use std::collections::HashMap;
use time::OffsetDateTime;

/// One participant's contribution to a session, for a summary view.
#[derive(Clone, Debug)]
pub struct ParticipantSummary {
    pub participant_id: ParticipantId,
    pub actor_id: ActorId,
    pub connected_seconds: i64,
    /// How many intervals were superseded by a correction, so a reader knows
    /// the total has been adjusted.
    pub corrected_intervals: u32,
}

/// A session's participation summary.
#[derive(Clone, Debug)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub participants: Vec<ParticipantSummary>,
}

#[derive(Debug, Clone)]
pub struct SessionParticipation {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Manager-only: every participant's connected time in one session.
///
/// Manager-gated because the summary exposes another actor's activity, which is
/// roster-level information. The session is loaded first so a caller
/// cannot tell a missing session from an invisible one.
pub async fn session_participation<P, S, R>(
    participation: &P,
    sessions: &S,
    rooms: &R,
    command: SessionParticipation,
    now: OffsetDateTime,
) -> Result<SessionSummary, ParticipationError>
where
    P: ParticipationStore + ?Sized,
    S: SessionStore + ?Sized,
    R: crate::ports::RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(ParticipationError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;

    let intervals = participation.list_for_session(command.session_id).await?;

    // The session end stops the clock for anything still open.
    let closes_at = session.ended_at;
    let mut participants: Vec<ParticipantSummary> = Vec::new();
    let mut participant_indexes = HashMap::new();
    for interval in &intervals {
        let index = *participant_indexes
            .entry(interval.participant_id)
            .or_insert_with(|| {
                let index = participants.len();
                participants.push(ParticipantSummary {
                    participant_id: interval.participant_id,
                    actor_id: interval.actor_id,
                    connected_seconds: 0,
                    corrected_intervals: 0,
                });
                index
            });
        let entry = &mut participants[index];
        if interval.is_superseded() {
            entry.corrected_intervals += 1;
            continue;
        }
        let end = interval.ended_at.or(closes_at).unwrap_or(now);
        entry.connected_seconds += (end - interval.started_at).whole_seconds().max(0);
    }

    Ok(SessionSummary {
        session_id: command.session_id,
        participants,
    })
}

#[derive(Debug, Clone)]
pub struct ParticipantParticipation {
    pub actor: ActorId,
    pub session_id: SessionId,
    pub participant_id: ParticipantId,
}

/// One participant's intervals and total, with the correction history.
///
/// Readable by the participant itself, so a client can show its own connected
/// time without seeing anyone else's. A manager may read any participant in a
/// room they manage.
pub async fn participant_participation<P, S, R>(
    participation: &P,
    sessions: &S,
    rooms: &R,
    command: ParticipantParticipation,
    now: OffsetDateTime,
) -> Result<(Vec<ParticipationInterval>, i64), ParticipationError>
where
    P: ParticipationStore + ?Sized,
    S: SessionStore + ?Sized,
    R: crate::ports::RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(ParticipationError::NotFound)?;

    let intervals = participation
        .list_for_participant(command.participant_id)
        .await?;
    // A participant with no interval in the session the caller named does not
    // exist as far as this query is concerned.
    if !intervals
        .iter()
        .any(|interval| interval.session_id == command.session_id)
    {
        return Err(ParticipationError::NotFound);
    }

    let is_self = intervals
        .iter()
        .any(|interval| interval.actor_id == command.actor);
    if !is_self {
        require_manager(rooms, session.room_id, command.actor).await?;
    }

    let total = total_connected_seconds(&intervals, now, session.ended_at);
    Ok((intervals, total))
}
