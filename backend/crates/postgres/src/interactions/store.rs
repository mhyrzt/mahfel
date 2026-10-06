use super::{access, activities, hands, snapshot};
use crate::stores::sqlx_store_error;
use mahfel_application::ports::{
    BoxFuture, InteractionCommand, InteractionError, InteractionSnapshot, InteractionStore,
    Principal, RaisedHand,
};
use mahfel_domain::{ParticipantId, SessionId};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PostgresInteractionStore {
    pool: PgPool,
}
impl PostgresInteractionStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
impl InteractionStore for PostgresInteractionStore {
    fn can_respond<'a>(
        &'a self,
        session: SessionId,
        principal: &'a Principal,
    ) -> BoxFuture<'a, Result<bool, InteractionError>> {
        Box::pin(async move {
            let mut connection = self.pool.acquire().await.map_err(sqlx_store_error)?;
            let access = access::read(&mut connection, session, principal).await?;
            Ok(access.live && access.participant.is_some())
        })
    }
    fn snapshot<'a>(
        &'a self,
        session: SessionId,
        principal: &'a Principal,
    ) -> BoxFuture<'a, Result<InteractionSnapshot, InteractionError>> {
        Box::pin(snapshot::read(&self.pool, session, principal))
    }
    fn execute<'a>(
        &'a self,
        session: SessionId,
        principal: &'a Principal,
        request_id: uuid::Uuid,
        command: InteractionCommand,
    ) -> BoxFuture<'a, Result<(), InteractionError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            access::lock(&mut tx, session).await?;
            access::lock_room_access(&mut tx, session, principal).await?;
            let access = access::read(&mut tx, session, principal).await?;
            if let Some(participant) = access.participant {
                let active: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM mahfel.session_participants WHERE id = $1 AND status = 'active' FOR SHARE")
                    .bind(participant.as_uuid()).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
                if active.is_none() {
                    return Err(InteractionError::Forbidden);
                }
            }
            let fingerprint = serde_json::to_value(&command)
                .map_err(mahfel_application::ports::StoreError::unexpected)?;
            let previous: Option<serde_json::Value> = sqlx::query_scalar("SELECT command FROM mahfel.interaction_commands WHERE session_id = $1 AND actor_id = $2 AND id = $3")
                .bind(session.as_uuid()).bind(principal.actor_id.as_uuid()).bind(request_id).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            if let Some(previous) = previous {
                if previous == fingerprint {
                    return Ok(());
                }
                return Err(InteractionError::Conflict(
                    "request identifier reused for a different command",
                ));
            }
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM mahfel.interaction_commands WHERE session_id = $1 AND actor_id = $2")
                .bind(session.as_uuid()).bind(principal.actor_id.as_uuid()).fetch_one(&mut *tx).await.map_err(sqlx_store_error)?;
            if count >= 2000 {
                return Err(InteractionError::Conflict("session command limit reached"));
            }
            let changed = match command {
                InteractionCommand::SetHand {
                    raised,
                    expected_version,
                } => {
                    hands::set(
                        &mut tx,
                        session,
                        principal.actor_id,
                        &access,
                        raised,
                        expected_version,
                    )
                    .await?
                }
                InteractionCommand::LowerHand {
                    participant_id,
                    expected_order,
                } => {
                    hands::lower(&mut tx, session, &access, participant_id, expected_order).await?
                }
                InteractionCommand::ClearHands { expected_version } => {
                    hands::clear(&mut tx, session, &access, expected_version).await?
                }
                command => {
                    activities::execute(&mut tx, session, principal.actor_id, &access, command)
                        .await?
                }
            };
            if changed {
                access::bump(&mut tx, session).await?;
            }
            sqlx::query("INSERT INTO mahfel.interaction_commands (session_id, actor_id, id, command) VALUES ($1, $2, $3, $4)")
                .bind(session.as_uuid()).bind(principal.actor_id.as_uuid()).bind(request_id).bind(fingerprint).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
    fn version(&self, session: SessionId) -> BoxFuture<'_, Result<i64, InteractionError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT interaction_version FROM mahfel.sessions WHERE id = $1")
                .bind(session.as_uuid())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| sqlx_store_error(e).into())
        })
    }
    fn clear_departed(
        &self,
        session: SessionId,
        participant: ParticipantId,
    ) -> BoxFuture<'_, Result<(), InteractionError>> {
        Box::pin(hands::depart(&self.pool, session, participant))
    }
    fn raised_hands(
        &self,
        session: SessionId,
    ) -> BoxFuture<'_, Result<Vec<RaisedHand>, InteractionError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            hands::list(&mut tx, session).await
        })
    }
}
