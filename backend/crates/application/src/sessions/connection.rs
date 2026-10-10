use super::SessionOperationError;
use crate::ports::{BlockStore, ParticipantStore, RoomStore, SessionStore};
use mahfel_domain::{ActorId, Participant, ParticipantId, RoomId, SessionId, SessionStatus};

#[derive(Clone, Copy)]
pub struct SessionConnection {
    pub actor: ActorId,
    pub participant: ParticipantId,
    pub session: SessionId,
    pub room: RoomId,
}

/// Revalidate durable admission before a connected socket operates media resources.
pub async fn authorize_connection<P, S, R, B>(
    participants: &P,
    sessions: &S,
    rooms: &R,
    blocks: &B,
    connection: SessionConnection,
) -> Result<Participant, SessionOperationError>
where
    P: ParticipantStore + ?Sized,
    S: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
    B: BlockStore + ?Sized,
{
    let participant = participants
        .get_active(connection.session, connection.actor)
        .await?
        .filter(|participant| participant.id == connection.participant)
        .ok_or(SessionOperationError::NotFound)?;
    let session = sessions
        .get(connection.session)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if session.status != SessionStatus::Live || session.room_id != connection.room {
        return Err(SessionOperationError::NotFound);
    }
    super::require_joinable(rooms, connection.room, connection.actor).await?;
    super::require_not_blocked(
        blocks,
        connection.room,
        connection.session,
        connection.actor,
    )
    .await?;
    Ok(participant)
}
