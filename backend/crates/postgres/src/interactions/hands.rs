use super::access::{self, Access};
use crate::stores::sqlx_store_error;
use mahfel_application::ports::{InteractionError, RaisedHand};
use mahfel_domain::{ActorId, ParticipantId, SessionId};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub(super) async fn set(
    connection: &mut PgConnection,
    session: SessionId,
    actor: ActorId,
    access: &Access,
    raised: bool,
    expected: i64,
) -> Result<bool, InteractionError> {
    if !access.live {
        return Err(InteractionError::Conflict("session is not live"));
    }
    let participant = access.participant.ok_or(InteractionError::Forbidden)?;
    let own: Option<i64> = sqlx::query_scalar(
        "SELECT queue_order FROM mahfel.raised_hands WHERE session_id = $1 AND actor_id = $2",
    )
    .bind(session.as_uuid())
    .bind(actor.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(sqlx_store_error)?;
    if raised {
        if own.is_some() {
            return Ok(false);
        }
        if expected != 0 {
            return Err(InteractionError::Conflict("hand state changed"));
        }
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM mahfel.raised_hands WHERE session_id = $1")
                .bind(session.as_uuid())
                .fetch_one(&mut *connection)
                .await
                .map_err(sqlx_store_error)?;
        if count >= 2000 {
            return Err(InteractionError::Conflict("hand queue limit reached"));
        }
        sqlx::query("INSERT INTO mahfel.raised_hands (session_id, participant_id, actor_id) VALUES ($1, $2, $3)")
            .bind(session.as_uuid()).bind(participant.as_uuid()).bind(actor.as_uuid()).execute(connection).await.map_err(sqlx_store_error)?;
        Ok(true)
    } else {
        let result = sqlx::query("DELETE FROM mahfel.raised_hands WHERE session_id = $1 AND actor_id = $2 AND queue_order = $3")
            .bind(session.as_uuid()).bind(actor.as_uuid()).bind(expected).execute(connection).await.map_err(sqlx_store_error)?;
        Ok(result.rows_affected() != 0)
    }
}
pub(super) async fn lower(
    connection: &mut PgConnection,
    session: SessionId,
    access: &Access,
    participant: ParticipantId,
    order: i64,
) -> Result<bool, InteractionError> {
    if !access.live || !access.can_moderate {
        return Err(InteractionError::Forbidden);
    }
    Ok(sqlx::query("DELETE FROM mahfel.raised_hands WHERE session_id = $1 AND participant_id = $2 AND queue_order = $3")
        .bind(session.as_uuid()).bind(participant.as_uuid()).bind(order).execute(connection).await.map_err(sqlx_store_error)?.rows_affected() != 0)
}
pub(super) async fn clear(
    connection: &mut PgConnection,
    session: SessionId,
    access: &Access,
    version: i64,
) -> Result<bool, InteractionError> {
    if !access.live || !access.can_moderate {
        return Err(InteractionError::Forbidden);
    }
    if access.version != version {
        return Err(InteractionError::Conflict(
            "queue changed; refresh before clearing",
        ));
    }
    Ok(
        sqlx::query("DELETE FROM mahfel.raised_hands WHERE session_id = $1")
            .bind(session.as_uuid())
            .execute(connection)
            .await
            .map_err(sqlx_store_error)?
            .rows_affected()
            != 0,
    )
}
pub(super) async fn depart(
    pool: &PgPool,
    session: SessionId,
    participant: ParticipantId,
) -> Result<(), InteractionError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    access::lock(&mut tx, session).await?;
    let changed = sqlx::query(
        "DELETE FROM mahfel.raised_hands WHERE session_id = $1 AND participant_id = $2",
    )
    .bind(session.as_uuid())
    .bind(participant.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(sqlx_store_error)?
    .rows_affected()
        != 0;
    if changed {
        access::bump(&mut tx, session).await?;
    }
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(())
}
#[derive(sqlx::FromRow)]
struct HandRow {
    participant_id: Uuid,
    actor_id: Uuid,
    name: String,
    queue_order: i64,
}
pub(super) async fn list(
    connection: &mut PgConnection,
    session: SessionId,
) -> Result<Vec<RaisedHand>, InteractionError> {
    let rows: Vec<HandRow> = sqlx::query_as("SELECT h.participant_id, h.actor_id, h.queue_order, COALESCE(u.display_name, sp.display_name, sp.name, 'Participant') AS name FROM mahfel.raised_hands h LEFT JOIN mahfel.users u ON u.id = h.actor_id LEFT JOIN mahfel.service_principals sp ON sp.id = h.actor_id WHERE h.session_id = $1 ORDER BY h.queue_order LIMIT 2000")
        .bind(session.as_uuid()).fetch_all(connection).await.map_err(sqlx_store_error)?;
    Ok(rows
        .into_iter()
        .map(|h| RaisedHand {
            participant_id: ParticipantId::from_uuid(h.participant_id),
            actor_id: ActorId::from_uuid(h.actor_id),
            name: h.name,
            order: h.queue_order,
        })
        .collect())
}
