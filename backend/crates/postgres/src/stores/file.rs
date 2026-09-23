use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, FileStore, StoreError};
use mahfel_domain::{
    ActorId, ChatMessageId, FileContext, FileId, FileObject, FileStatus, MAX_OWNER_FILE_BYTES,
    RoomId, SessionId,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresFileStore {
    pool: PgPool,
}

impl PostgresFileStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FileStore for PostgresFileStore {
    fn context_room(
        &self,
        context: FileContext,
    ) -> BoxFuture<'_, Result<Option<RoomId>, StoreError>> {
        Box::pin(async move {
            let id = match context {
                FileContext::Room(id) => return Ok(Some(id)),
                FileContext::Session(id) | FileContext::Recording(id) => id.as_uuid(),
                FileContext::ChatMessage(id) => id.as_uuid(),
                FileContext::Avatar(_) => return Ok(None),
            };
            let query = match context {
                FileContext::Session(_) | FileContext::Recording(_) => {
                    "SELECT room_id FROM mahfel.sessions WHERE id = $1"
                }
                FileContext::ChatMessage(_) => {
                    "SELECT channel.room_id FROM mahfel.chat_messages message JOIN mahfel.chat_channels channel ON channel.id = message.channel_id WHERE message.id = $1 AND message.deleted_at IS NULL"
                }
                _ => return Ok(None),
            };
            sqlx::query_scalar::<_, Uuid>(query)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map(|room| room.map(RoomId::from_uuid))
                .map_err(sqlx_store_error)
        })
    }

    fn insert<'a>(&'a self, file: &'a FileObject) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let (room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id) =
                context_columns(file.context);
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let owner: Option<Uuid> =
                sqlx::query_scalar("SELECT id FROM mahfel.actors WHERE id = $1 FOR UPDATE")
                    .bind(file.owner.as_uuid())
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(sqlx_store_error)?;
            if owner.is_none() {
                return Err(StoreError::MissingReference("file owner".into()));
            }
            let used: i64 = sqlx::query_scalar("SELECT COALESCE(sum(size_bytes), 0)::BIGINT FROM mahfel.files WHERE owner_actor_id = $1 AND status IN ('pending','quarantined','ready')")
                .bind(file.owner.as_uuid()).fetch_one(&mut *tx).await.map_err(sqlx_store_error)?;
            if u64::try_from(used)
                .map_err(store_error)?
                .saturating_add(file.size_bytes)
                > MAX_OWNER_FILE_BYTES
            {
                return Err(StoreError::Conflict("file quota exceeded".into()));
            }
            sqlx::query("INSERT INTO mahfel.files (id, object_key, owner_actor_id, room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id, status, file_name, declared_mime, size_bytes, checksum_sha_256, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)")
                .bind(file.id.as_uuid()).bind(&file.object_key).bind(file.owner.as_uuid())
                .bind(room_id).bind(session_id).bind(chat_message_id).bind(avatar_actor_id).bind(recording_session_id)
                .bind(file.status.as_str()).bind(&file.file_name).bind(&file.declared_mime)
                .bind(i64::try_from(file.size_bytes).map_err(store_error)?).bind(&file.checksum_sha256)
                .bind(file.created_at).bind(file.updated_at)
                .execute(&mut *tx).await.map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn get(&self, id: FileId) -> BoxFuture<'_, Result<Option<FileObject>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, FileRow>(FILE_SELECT_BY_ID)
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(TryInto::try_into).transpose()
        })
    }

    fn list(
        &self,
        context: FileContext,
        limit: u32,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<FileObject>, StoreError>> {
        Box::pin(async move {
            let (room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id) =
                context_columns(context);
            let rows = sqlx::query_as::<_, FileRow>("SELECT id, object_key, owner_actor_id, room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id, status, file_name, declared_mime, verified_mime, size_bytes, checksum_sha_256, created_at, updated_at FROM mahfel.files WHERE status = 'ready' AND (room_id = $1 OR session_id = $2 OR chat_message_id = $3 OR avatar_actor_id = $4 OR recording_session_id = $5) ORDER BY created_at DESC, id DESC LIMIT $6 OFFSET $7")
                .bind(room_id).bind(session_id).bind(chat_message_id).bind(avatar_actor_id).bind(recording_session_id)
                .bind(i64::from(limit)).bind(i64::try_from(offset).unwrap_or(i64::MAX))
                .fetch_all(&self.pool).await.map_err(sqlx_store_error)?;
            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn transition<'a>(
        &'a self,
        id: FileId,
        from: FileStatus,
        to: FileStatus,
        verified_mime: Option<&'a str>,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        Box::pin(async move {
            let result = sqlx::query("UPDATE mahfel.files SET status = $3, verified_mime = COALESCE($4, verified_mime), updated_at = $5 WHERE id = $1 AND status = $2")
                .bind(id.as_uuid()).bind(from.as_str()).bind(to.as_str()).bind(verified_mime).bind(now)
                .execute(&self.pool).await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn pending_before(
        &self,
        before: OffsetDateTime,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<FileObject>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, FileRow>("SELECT id, object_key, owner_actor_id, room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id, status, file_name, declared_mime, verified_mime, size_bytes, checksum_sha_256, created_at, updated_at FROM mahfel.files WHERE status IN ('pending','quarantined','rejected','deleting') AND updated_at < $1 ORDER BY updated_at LIMIT $2")
                .bind(before).bind(i64::from(limit)).fetch_all(&self.pool).await.map_err(sqlx_store_error)?;
            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn remove(&self, id: FileId) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query("DELETE FROM mahfel.files WHERE id = $1 AND status = 'deleting'")
                .bind(id.as_uuid())
                .execute(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn publish<'a>(
        &'a self,
        id: FileId,
        key: &'a str,
        mime: &'a str,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        Box::pin(async move {
            let result = sqlx::query("UPDATE mahfel.files SET object_key = $2, verified_mime = $3, status = 'ready', updated_at = $4 WHERE id = $1 AND status = 'quarantined'")
                .bind(id.as_uuid()).bind(key).bind(mime).bind(now)
                .execute(&self.pool).await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn record_download(
        &self,
        id: FileId,
        actor: ActorId,
        now: OffsetDateTime,
    ) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query("INSERT INTO mahfel.file_download_audit (id, file_id, actor_id, created_at) VALUES ($1,$2,$3,$4)")
                .bind(Uuid::new_v4()).bind(id.as_uuid()).bind(actor.as_uuid()).bind(now)
                .execute(&self.pool).await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
}

const FILE_SELECT_BY_ID: &str = "SELECT id, object_key, owner_actor_id, room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id, status, file_name, declared_mime, verified_mime, size_bytes, checksum_sha_256, created_at, updated_at FROM mahfel.files WHERE id = $1";

#[derive(FromRow)]
struct FileRow {
    id: Uuid,
    object_key: String,
    owner_actor_id: Uuid,
    room_id: Option<Uuid>,
    session_id: Option<Uuid>,
    chat_message_id: Option<Uuid>,
    avatar_actor_id: Option<Uuid>,
    recording_session_id: Option<Uuid>,
    status: String,
    file_name: String,
    declared_mime: String,
    verified_mime: Option<String>,
    size_bytes: i64,
    #[sqlx(rename = "checksum_sha_256")]
    checksum_sha256: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl TryFrom<FileRow> for FileObject {
    type Error = StoreError;

    fn try_from(row: FileRow) -> Result<Self, Self::Error> {
        let context = if let Some(id) = row.room_id {
            FileContext::Room(RoomId::from_uuid(id))
        } else if let Some(id) = row.session_id {
            FileContext::Session(SessionId::from_uuid(id))
        } else if let Some(id) = row.chat_message_id {
            FileContext::ChatMessage(ChatMessageId::from_uuid(id))
        } else if let Some(id) = row.avatar_actor_id {
            FileContext::Avatar(ActorId::from_uuid(id))
        } else if let Some(id) = row.recording_session_id {
            FileContext::Recording(SessionId::from_uuid(id))
        } else {
            return Err(StoreError::Unexpected("file has no context".into()));
        };
        Ok(Self {
            id: FileId::from_uuid(row.id),
            object_key: row.object_key,
            owner: ActorId::from_uuid(row.owner_actor_id),
            context,
            status: FileStatus::from_db(&row.status)
                .ok_or_else(|| StoreError::Unexpected("invalid file status".into()))?,
            file_name: row.file_name,
            declared_mime: row.declared_mime,
            verified_mime: row.verified_mime,
            size_bytes: u64::try_from(row.size_bytes).map_err(store_error)?,
            checksum_sha256: row.checksum_sha256,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

type ContextColumns = (
    Option<Uuid>,
    Option<Uuid>,
    Option<Uuid>,
    Option<Uuid>,
    Option<Uuid>,
);

fn context_columns(context: FileContext) -> ContextColumns {
    match context {
        FileContext::Room(id) => (Some(id.as_uuid()), None, None, None, None),
        FileContext::Session(id) => (None, Some(id.as_uuid()), None, None, None),
        FileContext::ChatMessage(id) => (None, None, Some(id.as_uuid()), None, None),
        FileContext::Avatar(id) => (None, None, None, Some(id.as_uuid()), None),
        FileContext::Recording(id) => (None, None, None, None, Some(id.as_uuid())),
    }
}
