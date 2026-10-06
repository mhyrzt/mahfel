use super::{access, activities::ActivityRow, hands};
use crate::stores::sqlx_store_error;
use mahfel_application::ports::{
    ActivityRecord, ActivityVote, InteractionError, InteractionSnapshot, Principal,
};
use mahfel_domain::{ActorId, SessionId};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct VoteRow {
    activity_id: Uuid,
    actor_id: Uuid,
    name: String,
    selected: Vec<i16>,
    version: i64,
}

pub(super) async fn read(
    pool: &PgPool,
    session: SessionId,
    principal: &Principal,
) -> Result<InteractionSnapshot, InteractionError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(sqlx_store_error)?;
    let access = access::read(&mut tx, session, principal).await?;
    let rows: Vec<ActivityRow> = sqlx::query_as("SELECT id, definition, status FROM mahfel.session_activities WHERE session_id = $1 AND (status <> 'draft' OR $2) ORDER BY created_at DESC, id LIMIT 50")
        .bind(session.as_uuid()).bind(access.can_manage).fetch_all(&mut *tx).await.map_err(sqlx_store_error)?;
    let votes: Vec<VoteRow> = sqlx::query_as("SELECT v.activity_id, v.actor_id, v.selected, v.vote_version AS version, COALESCE(u.display_name, sp.display_name, sp.name, 'Participant') AS name FROM mahfel.activity_votes v JOIN mahfel.session_activities a ON a.id = v.activity_id LEFT JOIN mahfel.users u ON u.id = v.actor_id LEFT JOIN mahfel.service_principals sp ON sp.id = v.actor_id WHERE a.session_id = $1 ORDER BY v.actor_id LIMIT 100000")
        .bind(session.as_uuid()).fetch_all(&mut *tx).await.map_err(sqlx_store_error)?;
    let activities = rows
        .into_iter()
        .map(|row| {
            let status = row.status()?;
            Ok(ActivityRecord {
                id: row.id,
                status,
                definition: row.definition.0,
                votes: votes
                    .iter()
                    .filter(|v| v.activity_id == row.id)
                    .map(|v| ActivityVote {
                        actor_id: ActorId::from_uuid(v.actor_id),
                        name: v.name.clone(),
                        selected: v.selected.iter().map(|n| *n as u16).collect(),
                        version: v.version,
                    })
                    .collect(),
            })
        })
        .collect::<Result<_, InteractionError>>()?;
    let hands = hands::list(&mut tx, session).await?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(InteractionSnapshot {
        version: access.version,
        live: access.live,
        can_manage: access.can_manage,
        can_moderate: access.can_moderate,
        can_respond: access.live && access.participant.is_some(),
        activities,
        hands,
    })
}
