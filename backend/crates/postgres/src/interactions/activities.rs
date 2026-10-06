use super::access::Access;
use crate::stores::sqlx_store_error;
use mahfel_application::ports::{ActivityStatus, InteractionCommand, InteractionError};
use mahfel_domain::{ActivityDefinition, ActivityKind, ActorId, SessionId};
use sqlx::{PgConnection, types::Json};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub(super) struct ActivityRow {
    pub id: Uuid,
    pub definition: Json<ActivityDefinition>,
    pub status: String,
}
impl ActivityRow {
    pub fn status(&self) -> Result<ActivityStatus, InteractionError> {
        match self.status.as_str() {
            "draft" => Ok(ActivityStatus::Draft),
            "open" => Ok(ActivityStatus::Open),
            "closed" => Ok(ActivityStatus::Closed),
            _ => Err(InteractionError::Conflict("invalid activity state")),
        }
    }
}
async fn get(
    connection: &mut PgConnection,
    session: SessionId,
    id: Uuid,
) -> Result<ActivityRow, InteractionError> {
    sqlx::query_as("SELECT id, definition, status FROM mahfel.session_activities WHERE session_id = $1 AND id = $2")
        .bind(session.as_uuid()).bind(id).fetch_optional(connection).await.map_err(sqlx_store_error)?.ok_or(InteractionError::NotFound)
}

pub(super) async fn execute(
    connection: &mut PgConnection,
    session: SessionId,
    actor: ActorId,
    access: &Access,
    command: InteractionCommand,
) -> Result<bool, InteractionError> {
    match command {
        InteractionCommand::SaveDraft { id, definition } => {
            if !access.can_manage {
                return Err(InteractionError::Forbidden);
            }
            if !access.can_prepare {
                return Err(InteractionError::Conflict("session has ended"));
            }
            let definition = definition.normalize()?;
            let existing: Option<(Json<ActivityDefinition>, String)> = sqlx::query_as("SELECT definition, status FROM mahfel.session_activities WHERE session_id = $1 AND id = $2")
                .bind(session.as_uuid()).bind(id).fetch_optional(&mut *connection).await.map_err(sqlx_store_error)?;
            if let Some((previous, status)) = existing {
                if previous.0 == definition {
                    return Ok(false);
                }
                if status != "draft" {
                    return Err(InteractionError::Conflict(
                        "launched activities cannot be edited",
                    ));
                }
                sqlx::query("UPDATE mahfel.session_activities SET definition = $3 WHERE session_id = $1 AND id = $2")
                    .bind(session.as_uuid()).bind(id).bind(Json(definition)).execute(connection).await.map_err(sqlx_store_error)?;
            } else {
                let count: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM mahfel.session_activities WHERE session_id = $1",
                )
                .bind(session.as_uuid())
                .fetch_one(&mut *connection)
                .await
                .map_err(sqlx_store_error)?;
                if count >= 50 {
                    return Err(InteractionError::Conflict("session activity limit reached"));
                }
                sqlx::query("INSERT INTO mahfel.session_activities (id, session_id, created_by, definition) VALUES ($1, $2, $3, $4)")
                    .bind(id).bind(session.as_uuid()).bind(actor.as_uuid()).bind(Json(definition)).execute(connection).await.map_err(sqlx_store_error)?;
            }
            Ok(true)
        }
        InteractionCommand::Launch { id } | InteractionCommand::Close { id } => {
            if !access.can_manage {
                return Err(InteractionError::Forbidden);
            }
            if !access.live {
                return Err(InteractionError::Conflict("session is not live"));
            }
            let row = get(connection, session, id).await?;
            let launch = matches!(command, InteractionCommand::Launch { .. });
            if row.status == if launch { "open" } else { "closed" } {
                return Ok(false);
            }
            if row.status != if launch { "draft" } else { "open" } {
                return Err(InteractionError::Conflict("activity state changed"));
            }
            if launch {
                let open: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.session_activities WHERE session_id = $1 AND status = 'open')")
                    .bind(session.as_uuid()).fetch_one(&mut *connection).await.map_err(sqlx_store_error)?;
                if open {
                    return Err(InteractionError::Conflict(
                        "close the active activity first",
                    ));
                }
            }
            sqlx::query("UPDATE mahfel.session_activities SET status = $3, closed_at = CASE WHEN $3 = 'closed' THEN now() ELSE NULL END WHERE session_id = $1 AND id = $2")
                .bind(session.as_uuid()).bind(id).bind(if launch { "open" } else { "closed" }).execute(connection).await.map_err(sqlx_store_error)?;
            Ok(true)
        }
        InteractionCommand::Vote {
            id,
            mut selected,
            expected_version,
        } => {
            if !access.live || access.participant.is_none() {
                return Err(InteractionError::Forbidden);
            }
            let row = get(connection, session, id).await?;
            row.definition.validate_selection(&selected)?;
            selected.sort_unstable();
            let selected: Vec<i16> = selected.into_iter().map(|n| n as i16).collect();
            let own: Option<(Vec<i16>, i64)> = sqlx::query_as("SELECT selected, vote_version AS version FROM mahfel.activity_votes WHERE activity_id = $1 AND actor_id = $2")
                .bind(id).bind(actor.as_uuid()).fetch_optional(&mut *connection).await.map_err(sqlx_store_error)?;
            if own.as_ref().is_some_and(|(previous, version)| {
                previous == &selected
                    && (*version == expected_version
                        || Some(*version) == expected_version.checked_add(1))
            }) {
                return Ok(false);
            }
            if row.status != "open" {
                return Err(InteractionError::Conflict("activity is closed"));
            }
            if own
                .as_ref()
                .is_some_and(|(_, version)| *version != expected_version)
                || (own.is_none() && expected_version != 0)
            {
                return Err(InteractionError::Conflict(
                    "your submission changed on another device",
                ));
            }
            if row.definition.kind == ActivityKind::Quiz
                && own
                    .as_ref()
                    .is_some_and(|(selected, _)| !selected.is_empty())
            {
                return Err(InteractionError::Conflict("quiz answer is final"));
            }
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM mahfel.activity_votes WHERE activity_id = $1",
            )
            .bind(id)
            .fetch_one(&mut *connection)
            .await
            .map_err(sqlx_store_error)?;
            if own.is_none() && count >= 2000 {
                return Err(InteractionError::Conflict(
                    "activity response limit reached",
                ));
            }
            sqlx::query("INSERT INTO mahfel.activity_votes AS vote (activity_id, actor_id, selected, vote_version) VALUES ($1, $2, $3, 1) ON CONFLICT (activity_id, actor_id) DO UPDATE SET selected = EXCLUDED.selected, vote_version = vote.vote_version + 1")
                .bind(id).bind(actor.as_uuid()).bind(selected).execute(connection).await.map_err(sqlx_store_error)?;
            Ok(true)
        }
        InteractionCommand::Retract {
            id,
            expected_version,
        } => {
            if !access.live || access.participant.is_none() {
                return Err(InteractionError::Forbidden);
            }
            let row = get(connection, session, id).await?;
            if row.status != "open" || row.definition.kind != ActivityKind::Poll {
                return Err(InteractionError::Conflict(
                    "only open polls allow retraction",
                ));
            }
            let result = sqlx::query("UPDATE mahfel.activity_votes SET selected = '{}', vote_version = vote_version + 1 WHERE activity_id = $1 AND actor_id = $2 AND vote_version = $3 AND cardinality(selected) > 0")
                .bind(id).bind(actor.as_uuid()).bind(expected_version).execute(connection).await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() != 0)
        }
        _ => Err(InteractionError::Conflict("invalid activity command")),
    }
}
