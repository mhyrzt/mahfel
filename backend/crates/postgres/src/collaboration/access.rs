use mahfel_application::ports::{
    CollaborationAccess, CollaborationParticipant, ParticipantAccessUpdate, StoreError,
};
use mahfel_domain::{ActorId, CollaborationEvent, CollaborationEventKind, SessionId};
use serde_json::json;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::stores::sqlx_store_error;

use super::events;

const ACCESS_SQL: &str = "\
SELECT session.status, membership.member_role, \
       EXISTS (SELECT 1 FROM mahfel.session_participants participant \
               WHERE participant.session_id = session.id \
                 AND participant.actor_id = $2 AND participant.status = 'active') AS admitted, \
       COALESCE(grant_row.can_edit, false) AS granted, \
       COALESCE(grant_row.can_publish_audio, true) AS can_publish_audio, \
       COALESCE(grant_row.can_publish_video, true) AS can_publish_video, \
       COALESCE(grant_row.can_share_screen, true) AS can_share_screen, \
       COALESCE(grant_row.can_present, false) AS can_present \
FROM mahfel.sessions session \
LEFT JOIN mahfel.room_memberships membership \
  ON membership.room_id = session.room_id AND membership.actor_id = $2 \
LEFT JOIN mahfel.collaboration_edit_grants grant_row \
  ON grant_row.session_id = session.id AND grant_row.actor_id = $2 \
WHERE session.id = $1";

#[derive(sqlx::FromRow)]
struct AccessRow {
    status: String,
    member_role: Option<String>,
    admitted: bool,
    granted: bool,
    can_publish_audio: bool,
    can_publish_video: bool,
    can_share_screen: bool,
    can_present: bool,
}

fn decide(row: Option<AccessRow>) -> CollaborationAccess {
    let Some(row) = row else {
        return CollaborationAccess {
            can_read: false,
            can_edit: false,
            can_publish_audio: false,
            can_publish_video: false,
            can_share_screen: false,
            can_present: false,
            can_manage: false,
            live: false,
        };
    };
    let member = row.member_role.is_some();
    let manager = row.member_role.as_deref() == Some("manager");
    let live = row.status == "live";
    let can_read = member && (row.status == "ended" || (live && (row.admitted || manager)));
    let can_manage = can_read && live && manager;
    CollaborationAccess {
        can_read,
        can_edit: can_read && live && (can_manage || row.granted),
        can_publish_audio: can_read && live && (can_manage || row.can_publish_audio),
        can_publish_video: can_read && live && (can_manage || row.can_publish_video),
        can_share_screen: can_read && live && (can_manage || row.can_share_screen),
        can_present: can_read && live && (can_manage || row.can_present),
        can_manage,
        live,
    }
}

pub(super) async fn access(
    pool: &PgPool,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<CollaborationAccess, StoreError> {
    let row = sqlx::query_as::<_, AccessRow>(ACCESS_SQL)
        .bind(session_id.as_uuid())
        .bind(actor_id.as_uuid())
        .fetch_optional(pool)
        .await
        .map_err(sqlx_store_error)?;
    Ok(decide(row))
}

pub(super) async fn access_in_tx(
    connection: &mut PgConnection,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<CollaborationAccess, StoreError> {
    let row = sqlx::query_as::<_, AccessRow>(ACCESS_SQL)
        .bind(session_id.as_uuid())
        .bind(actor_id.as_uuid())
        .fetch_optional(connection)
        .await
        .map_err(sqlx_store_error)?;
    Ok(decide(row))
}

pub(super) async fn lock_session(
    connection: &mut PgConnection,
    session_id: SessionId,
) -> Result<(), StoreError> {
    let found =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
            .bind(session_id.as_uuid())
            .fetch_optional(connection)
            .await
            .map_err(sqlx_store_error)?;
    found
        .map(|_| ())
        .ok_or_else(|| StoreError::NotFound("session".into()))
}

pub(super) async fn set_participant_access(
    pool: &PgPool,
    session_id: SessionId,
    manager: ActorId,
    target: ActorId,
    update: ParticipantAccessUpdate,
    event_id: Uuid,
) -> Result<CollaborationEvent, StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    lock_session(&mut tx, session_id).await?;
    if !access_in_tx(&mut tx, session_id, manager).await?.can_manage {
        return Err(StoreError::NotFound("collaboration management".into()));
    }
    if target == manager {
        return Err(StoreError::Conflict("cannot change own access".into()));
    }
    let target_access = access_in_tx(&mut tx, session_id, target).await?;
    if !target_access.can_read || target_access.can_manage {
        return Err(StoreError::NotFound("admitted participant".into()));
    }
    sqlx::query(
        "INSERT INTO mahfel.collaboration_edit_grants AS grant_row \
         (session_id, actor_id, can_edit, can_publish_audio, can_publish_video, \
          can_share_screen, can_present, updated_by) \
         VALUES ($1, $2, COALESCE($3, false), COALESCE($4, true), COALESCE($5, true), \
                 COALESCE($6, true), COALESCE($7, false), $8) \
         ON CONFLICT (session_id, actor_id) DO UPDATE \
         SET can_edit = COALESCE($3, grant_row.can_edit), \
             can_publish_audio = COALESCE($4, grant_row.can_publish_audio), \
             can_publish_video = COALESCE($5, grant_row.can_publish_video), \
             can_share_screen = COALESCE($6, grant_row.can_share_screen), \
             can_present = COALESCE($7, grant_row.can_present), \
             updated_by = EXCLUDED.updated_by, updated_at = now()",
    )
    .bind(session_id.as_uuid())
    .bind(target.as_uuid())
    .bind(update.can_edit)
    .bind(update.can_publish_audio)
    .bind(update.can_publish_video)
    .bind(update.can_share_screen)
    .bind(update.can_present)
    .bind(manager.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    let event = events::insert_in_tx(
        &mut tx,
        session_id,
        event_id,
        manager,
        None,
        CollaborationEventKind::EditGrant,
        None,
        &json!({"actor_id": target.to_string(), "can_edit": update.can_edit,
            "can_publish_audio": update.can_publish_audio,
            "can_publish_video": update.can_publish_video,
            "can_share_screen": update.can_share_screen,
            "can_present": update.can_present}),
    )
    .await?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(event)
}

#[derive(sqlx::FromRow)]
struct ParticipantRow {
    actor_id: Uuid,
    display_name: String,
    can_edit: bool,
    can_publish_audio: bool,
    can_publish_video: bool,
    can_share_screen: bool,
    can_present: bool,
    is_manager: bool,
}

pub(super) async fn participants(
    pool: &PgPool,
    session_id: SessionId,
    manager: ActorId,
) -> Result<Vec<CollaborationParticipant>, StoreError> {
    if !access(pool, session_id, manager).await?.can_manage {
        return Err(StoreError::NotFound("collaboration management".into()));
    }
    let rows = sqlx::query_as::<_, ParticipantRow>(
        "SELECT participant.actor_id, \
         COALESCE(member.display_name, service.display_name, service.name, 'Participant') AS display_name, \
         (membership.member_role = 'manager' OR COALESCE(grant_row.can_edit, false)) AS can_edit, \
         (membership.member_role = 'manager' OR COALESCE(grant_row.can_publish_audio, true)) AS can_publish_audio, \
         (membership.member_role = 'manager' OR COALESCE(grant_row.can_publish_video, true)) AS can_publish_video, \
         (membership.member_role = 'manager' OR COALESCE(grant_row.can_share_screen, true)) AS can_share_screen, \
         (membership.member_role = 'manager' OR COALESCE(grant_row.can_present, false)) AS can_present, \
         (membership.member_role = 'manager') AS is_manager \
         FROM mahfel.session_participants participant \
         JOIN mahfel.sessions session ON session.id = participant.session_id \
         JOIN mahfel.room_memberships membership \
           ON membership.room_id = session.room_id AND membership.actor_id = participant.actor_id \
         LEFT JOIN mahfel.users member ON member.id = participant.actor_id \
         LEFT JOIN mahfel.service_principals service ON service.id = participant.actor_id \
         LEFT JOIN mahfel.collaboration_edit_grants grant_row \
           ON grant_row.session_id = participant.session_id AND grant_row.actor_id = participant.actor_id \
         WHERE participant.session_id = $1 AND participant.status = 'active' \
         ORDER BY display_name, participant.actor_id",
    )
    .bind(session_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(sqlx_store_error)?;
    Ok(rows
        .into_iter()
        .map(|row| CollaborationParticipant {
            actor_id: ActorId::from_uuid(row.actor_id),
            display_name: row.display_name,
            can_edit: row.can_edit,
            can_publish_audio: row.can_publish_audio,
            can_publish_video: row.can_publish_video,
            can_share_screen: row.can_share_screen,
            can_present: row.can_present,
            is_manager: row.is_manager,
        })
        .collect())
}
