use mahfel_application::ports::{PresentationState, StoreError};
use mahfel_domain::{ActorId, FileId, SessionId};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::access;
use crate::stores::sqlx_store_error;

pub(super) async fn current(
    pool: &PgPool,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<Option<PresentationState>, StoreError> {
    if !access::access(pool, session_id, actor_id).await?.can_read {
        return Err(StoreError::NotFound("collaboration session".into()));
    }
    let row: Option<(Uuid, i32, i32)> = sqlx::query_as(
        "SELECT selection_event_id, current_page, total_pages FROM mahfel.collaboration_presentations \
         WHERE session_id = $1 AND selection_event_id IS NOT NULL",
    ).bind(session_id.as_uuid()).fetch_optional(pool).await.map_err(sqlx_store_error)?;
    let Some((event_id, current_page, total_pages)) = row else {
        return Ok(None);
    };
    let file_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT file_id FROM mahfel.collaboration_presentation_assets WHERE event_id = $1 ORDER BY ordinal",
    ).bind(event_id).fetch_all(pool).await.map_err(sqlx_store_error)?;
    Ok(Some(PresentationState {
        file_ids: file_ids.into_iter().map(FileId::from_uuid).collect(),
        current_page: u32::try_from(current_page)
            .map_err(|error| StoreError::Unexpected(error.to_string()))?,
        total_pages: u32::try_from(total_pages)
            .map_err(|error| StoreError::Unexpected(error.to_string()))?,
    }))
}

enum Action {
    Select { files: Vec<Uuid>, pages: i32 },
    Page(i32),
    Stop,
}

fn parse(data: &Value) -> Result<Action, StoreError> {
    match data.get("action").and_then(Value::as_str) {
        Some("select") => {
            let files = data
                .get("file_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| StoreError::Conflict("invalid presentation assets".into()))?;
            if files.is_empty() || files.len() > 50 {
                return Err(StoreError::Conflict("invalid presentation assets".into()));
            }
            let files = files
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .and_then(|value| Uuid::parse_str(value).ok())
                        .ok_or_else(|| StoreError::Conflict("invalid presentation asset".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let pages = data
                .get("total_pages")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .filter(|value| (1..=1000).contains(value))
                .ok_or_else(|| StoreError::Conflict("invalid presentation page count".into()))?;
            Ok(Action::Select { files, pages })
        }
        Some("page") => data
            .get("page")
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .filter(|value| *value > 0)
            .map(Action::Page)
            .ok_or_else(|| StoreError::Conflict("invalid presentation page".into())),
        Some("stop") => Ok(Action::Stop),
        _ => Err(StoreError::Conflict("invalid presentation action".into())),
    }
}

pub(super) async fn validate(
    tx: &mut Transaction<'_, Postgres>,
    session_id: SessionId,
    data: &Value,
) -> Result<(), StoreError> {
    match parse(data)? {
        Action::Select { files, pages } => {
            let mut mime = None;
            for file_id in &files {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT verified_mime FROM mahfel.files WHERE id = $1 AND status = 'ready' \
                     AND (session_id = $2 OR room_id = (SELECT room_id FROM mahfel.sessions WHERE id = $2)) \
                     FOR UPDATE",
                ).bind(file_id).bind(session_id.as_uuid()).fetch_optional(&mut **tx)
                    .await.map_err(sqlx_store_error)?;
                let Some((file_mime,)) = row else {
                    return Err(StoreError::NotFound("presentation file".into()));
                };
                if !matches!(
                    file_mime.as_str(),
                    "application/pdf" | "image/png" | "image/jpeg"
                ) {
                    return Err(StoreError::Conflict("unsupported presentation file".into()));
                }
                if mime.is_some_and(|previous| previous != (file_mime == "application/pdf")) {
                    return Err(StoreError::Conflict("mixed presentation files".into()));
                }
                mime = Some(file_mime == "application/pdf");
            }
            if mime == Some(true) && files.len() != 1 {
                return Err(StoreError::Conflict(
                    "presentation PDF must be one file".into(),
                ));
            }
            if mime == Some(false) && usize::try_from(pages).ok() != Some(files.len()) {
                return Err(StoreError::Conflict("image page count mismatch".into()));
            }
        }
        Action::Page(page) => {
            let total: Option<i32> = sqlx::query_scalar(
                "SELECT total_pages FROM mahfel.collaboration_presentations WHERE session_id = $1",
            )
            .bind(session_id.as_uuid())
            .fetch_optional(&mut **tx)
            .await
            .map_err(sqlx_store_error)?
            .flatten();
            if total.is_none_or(|total| page > total) {
                return Err(StoreError::Conflict("presentation page unavailable".into()));
            }
        }
        Action::Stop => {}
    }
    Ok(())
}

pub(super) async fn apply(
    tx: &mut Transaction<'_, Postgres>,
    session_id: SessionId,
    event_id: Uuid,
    data: &Value,
) -> Result<(), StoreError> {
    match parse(data)? {
        Action::Select { files, pages } => {
            for (ordinal, file_id) in files.into_iter().enumerate() {
                sqlx::query("INSERT INTO mahfel.collaboration_presentation_assets (event_id, ordinal, file_id) VALUES ($1,$2,$3)")
                    .bind(event_id).bind(i16::try_from(ordinal).map_err(|error| StoreError::Unexpected(error.to_string()))?)
                    .bind(file_id).execute(&mut **tx).await.map_err(sqlx_store_error)?;
            }
            sqlx::query("INSERT INTO mahfel.collaboration_presentations (session_id, selection_event_id, current_page, total_pages) \
                VALUES ($1,$2,1,$3) ON CONFLICT (session_id) DO UPDATE SET \
                selection_event_id = EXCLUDED.selection_event_id, current_page = 1, total_pages = EXCLUDED.total_pages")
                .bind(session_id.as_uuid()).bind(event_id).bind(pages)
                .execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
        Action::Page(page) => {
            sqlx::query("UPDATE mahfel.collaboration_presentations SET current_page = $2 WHERE session_id = $1")
                .bind(session_id.as_uuid()).bind(page).execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
        Action::Stop => {
            sqlx::query("INSERT INTO mahfel.collaboration_presentations (session_id) VALUES ($1) \
                ON CONFLICT (session_id) DO UPDATE SET selection_event_id = NULL, current_page = NULL, total_pages = NULL")
                .bind(session_id.as_uuid()).execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
    }
    Ok(())
}
