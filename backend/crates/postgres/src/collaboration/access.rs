use mahfel_application::ports::{CollaborationAccess, CollaborationParticipant, StoreError};
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
       COALESCE(grant_row.can_edit, false) AS granted \
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
}

fn decide(row: Option<AccessRow>) -> CollaborationAccess {
    let Some(row) = row else {
        return CollaborationAccess {
            can_read: false,
            can_edit: false,
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

pub(super) async fn set_edit_grant(
    pool: &PgPool,
    session_id: SessionId,
    manager: ActorId,
    target: ActorId,
    can_edit: bool,
    event_id: Uuid,
) -> Result<CollaborationEvent, StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    lock_session(&mut tx, session_id).await?;
    if !access_in_tx(&mut tx, session_id, manager).await?.can_manage {
        return Err(StoreError::NotFound("collaboration management".into()));
    }
    if !access_in_tx(&mut tx, session_id, target).await?.can_read {
        return Err(StoreError::NotFound("admitted participant".into()));
    }
    sqlx::query(
        "INSERT INTO mahfel.collaboration_edit_grants \
         (session_id, actor_id, can_edit, updated_by) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (session_id, actor_id) DO UPDATE \
         SET can_edit = EXCLUDED.can_edit, updated_by = EXCLUDED.updated_by, updated_at = now()",
    )
    .bind(session_id.as_uuid())
    .bind(target.as_uuid())
    .bind(can_edit)
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
        &json!({"actor_id": target.to_string(), "can_edit": can_edit}),
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
         COALESCE(grant_row.can_edit, false) AS can_edit \
         FROM mahfel.session_participants participant \
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
        })
        .collect())
}
