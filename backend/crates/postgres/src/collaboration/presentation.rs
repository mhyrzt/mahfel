use mahfel_application::ports::{PresentationState, StoreError};
use mahfel_domain::{ActorId, FileId, SessionId};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::access;
use crate::stores::sqlx_store_error;

type CurrentPresentationRow = (Uuid, i32, i32, Option<i64>, Option<bool>, Option<f64>);

pub(super) async fn current(
    pool: &PgPool,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<Option<PresentationState>, StoreError> {
    if !access::access(pool, session_id, actor_id).await?.can_read {
        return Err(StoreError::NotFound("collaboration session".into()));
    }
    let row: Option<CurrentPresentationRow> = sqlx::query_as(
        "SELECT selection_event_id, current_page, total_pages, playback_position_ms, \
         playback_paused, playback_rate FROM mahfel.collaboration_presentations \
         WHERE session_id = $1 AND selection_event_id IS NOT NULL",
    )
    .bind(session_id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(sqlx_store_error)?;
    let Some((event_id, current_page, total_pages, position_ms, paused, rate)) = row else {
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
        playback_position_ms: position_ms
            .map(|value| {
                u64::try_from(value).map_err(|error| StoreError::Unexpected(error.to_string()))
            })
            .transpose()?,
        playback_paused: paused,
        playback_rate: rate,
    }))
}

enum Action {
    Select {
        files: Vec<Uuid>,
        pages: i32,
    },
    Page(i32),
    Playback {
        position_ms: i64,
        paused: bool,
        rate: f64,
    },
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
        Some("playback") => {
            let position_ms = data
                .get("position_ms")
                .and_then(Value::as_i64)
                .filter(|value| *value >= 0)
                .ok_or_else(|| StoreError::Conflict("invalid playback position".into()))?;
            let paused = data
                .get("paused")
                .and_then(Value::as_bool)
                .ok_or_else(|| StoreError::Conflict("invalid playback state".into()))?;
            let rate = data
                .get("rate")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && (0.25..=4.0).contains(value))
                .ok_or_else(|| StoreError::Conflict("invalid playback rate".into()))?;
            Ok(Action::Playback {
                position_ms,
                paused,
                rate,
            })
        }
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
            let mut has_pdf = false;
            let mut has_image = false;
            let mut has_video = false;
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
                match file_mime.as_str() {
                    "application/pdf" => has_pdf = true,
                    "image/png" | "image/jpeg" => has_image = true,
                    "video/mp4" | "video/webm" => has_video = true,
                    _ => {
                        return Err(StoreError::Conflict("unsupported presentation file".into()));
                    }
                }
            }
            if has_pdf && (has_image || has_video || files.len() != 1) {
                return Err(StoreError::Conflict(
                    "presentation PDF must be one file".into(),
                ));
            }
            if has_video && (has_image || files.len() != 1 || pages != 1) {
                return Err(StoreError::Conflict(
                    "presentation video must be one file".into(),
                ));
            }
            if has_image && usize::try_from(pages).ok() != Some(files.len()) {
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
        Action::Playback { .. } => {
            let mime: Option<String> = sqlx::query_scalar(
                "SELECT file.verified_mime FROM mahfel.collaboration_presentations presentation \
                 INNER JOIN mahfel.collaboration_presentation_assets asset \
                   ON asset.event_id = presentation.selection_event_id \
                 INNER JOIN mahfel.files file ON file.id = asset.file_id \
                 WHERE presentation.session_id = $1 \
                 ORDER BY asset.ordinal LIMIT 1",
            )
            .bind(session_id.as_uuid())
            .fetch_optional(&mut **tx)
            .await
            .map_err(sqlx_store_error)?
            .flatten();
            let Some(mime) = mime else {
                return Err(StoreError::Conflict("no presentation".into()));
            };
            if !matches!(mime.as_str(), "video/mp4" | "video/webm") {
                return Err(StoreError::Conflict(
                    "playback requires a video presentation".into(),
                ));
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
                selection_event_id = EXCLUDED.selection_event_id, current_page = 1, total_pages = EXCLUDED.total_pages, \
                playback_position_ms = NULL, playback_paused = NULL, playback_rate = NULL")
                .bind(session_id.as_uuid()).bind(event_id).bind(pages)
                .execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
        Action::Page(page) => {
            sqlx::query("UPDATE mahfel.collaboration_presentations SET current_page = $2 WHERE session_id = $1")
                .bind(session_id.as_uuid()).bind(page).execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
        Action::Playback {
            position_ms,
            paused,
            rate,
        } => {
            sqlx::query(
                "UPDATE mahfel.collaboration_presentations SET playback_position_ms = $2, \
                playback_paused = $3, playback_rate = $4 WHERE session_id = $1",
            )
            .bind(session_id.as_uuid())
            .bind(position_ms)
            .bind(paused)
            .bind(rate)
            .execute(&mut **tx)
            .await
            .map_err(sqlx_store_error)?;
        }
        Action::Stop => {
            sqlx::query("INSERT INTO mahfel.collaboration_presentations (session_id) VALUES ($1) \
                ON CONFLICT (session_id) DO UPDATE SET selection_event_id = NULL, current_page = NULL, total_pages = NULL, \
                playback_position_ms = NULL, playback_paused = NULL, playback_rate = NULL")
                .bind(session_id.as_uuid()).execute(&mut **tx).await.map_err(sqlx_store_error)?;
        }
    }
    Ok(())
}
