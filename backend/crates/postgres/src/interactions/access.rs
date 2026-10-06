use crate::stores::sqlx_store_error;
use mahfel_application::{
    policy::ServerCapabilityResolver,
    ports::{Capability, CapabilityContext, CapabilityResolver, InteractionError, Principal},
};
use mahfel_domain::{ParticipantId, RoomRole, SessionId};
use sqlx::PgConnection;
use uuid::Uuid;

pub(super) struct Access {
    pub live: bool,
    pub can_prepare: bool,
    pub can_manage: bool,
    pub can_moderate: bool,
    pub participant: Option<ParticipantId>,
    pub version: i64,
}
#[derive(sqlx::FromRow)]
struct AccessRow {
    status: String,
    room_status: String,
    visibility: String,
    admitted: bool,
    member_role: Option<String>,
    participant: Option<Uuid>,
    blocked: bool,
    interaction_version: i64,
}

pub(super) async fn read(
    connection: &mut PgConnection,
    session: SessionId,
    principal: &Principal,
) -> Result<Access, InteractionError> {
    let row: AccessRow = sqlx::query_as(
        "SELECT s.status, r.status AS room_status, COALESCE(s.visibility, r.visibility) AS visibility, m.member_role, s.interaction_version, \
         EXISTS (SELECT 1 FROM mahfel.admission_requests a WHERE a.session_id = s.id AND a.actor_id = $2 AND a.status = 'admitted') AS admitted, \
         (SELECT id FROM mahfel.session_participants p WHERE p.session_id = s.id AND p.actor_id = $2 AND p.status = 'active' LIMIT 1) AS participant, \
         EXISTS (SELECT 1 FROM mahfel.blocks b WHERE b.actor_id = $2 AND b.lifted_at IS NULL AND (b.expires_at IS NULL OR b.expires_at > now()) \
         AND (b.session_id = s.id OR b.room_id = s.room_id)) AS blocked \
         FROM mahfel.sessions s JOIN mahfel.rooms r ON r.id = s.room_id LEFT JOIN mahfel.room_memberships m ON m.room_id = s.room_id AND m.actor_id = $2 WHERE s.id = $1")
        .bind(session.as_uuid()).bind(principal.actor_id.as_uuid()).fetch_optional(connection).await.map_err(sqlx_store_error)?.ok_or(InteractionError::NotFound)?;
    let role = row
        .member_role
        .as_deref()
        .and_then(|role| RoomRole::try_from(role).ok());
    let capabilities =
        ServerCapabilityResolver.resolve(principal, CapabilityContext::in_room(role));
    let can_manage = capabilities.has(Capability::ManageSession);
    let can_moderate = capabilities.has(Capability::MuteOthers);
    let live = row.status == "live";
    if row.blocked
        || !(can_manage || role.is_some() || row.visibility == "public" || row.admitted)
        || (row.status != "ended" && row.room_status != "active")
        || !(can_manage || (live && row.participant.is_some()) || row.status == "ended")
    {
        return Err(InteractionError::NotFound);
    }
    Ok(Access {
        live,
        can_prepare: row.status == "scheduled" || live,
        can_manage,
        can_moderate,
        participant: row.participant.map(ParticipantId::from_uuid),
        version: row.interaction_version,
    })
}

pub(super) async fn lock(
    connection: &mut PgConnection,
    session: SessionId,
) -> Result<(), InteractionError> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
        .bind(session.as_uuid())
        .fetch_optional(connection)
        .await
        .map_err(sqlx_store_error)?
        .ok_or(InteractionError::NotFound)?;
    Ok(())
}

pub(super) async fn lock_room_access(
    connection: &mut PgConnection,
    session: SessionId,
    principal: &Principal,
) -> Result<(), InteractionError> {
    sqlx::query("SELECT r.id FROM mahfel.rooms r JOIN mahfel.sessions s ON s.room_id = r.id WHERE s.id = $1 FOR SHARE OF r")
        .bind(session.as_uuid()).execute(&mut *connection).await.map_err(sqlx_store_error)?;
    sqlx::query("SELECT m.actor_id FROM mahfel.room_memberships m JOIN mahfel.sessions s ON s.room_id = m.room_id WHERE s.id = $1 AND m.actor_id = $2 FOR SHARE OF m")
        .bind(session.as_uuid()).bind(principal.actor_id.as_uuid()).execute(connection).await.map_err(sqlx_store_error)?;
    Ok(())
}
pub(super) async fn bump(
    connection: &mut PgConnection,
    session: SessionId,
) -> Result<(), InteractionError> {
    sqlx::query(
        "UPDATE mahfel.sessions SET interaction_version = interaction_version + 1 WHERE id = $1",
    )
    .bind(session.as_uuid())
    .execute(connection)
    .await
    .map_err(sqlx_store_error)?;
    Ok(())
}
