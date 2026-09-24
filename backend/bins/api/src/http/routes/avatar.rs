use crate::http::state::AppState;
use mahfel_domain::{ActorId, FileContext, FileStatus};
use std::time::Duration;

pub(super) async fn ready_avatar_url(state: &AppState, actor: ActorId) -> Option<String> {
    let files = match state.files.list(FileContext::Avatar(actor), 1, 0).await {
        Ok(files) => files,
        Err(error) => {
            tracing::warn!(%error, "avatar lookup failed");
            return None;
        }
    };
    let file = files
        .into_iter()
        .find(|file| file.status == FileStatus::Ready)?;
    match state
        .storage
        .presign_download(&file.object_key, &file.file_name, Duration::from_secs(3600))
        .await
    {
        Ok(signed) => Some(signed.url),
        Err(error) => {
            tracing::warn!(%error, "avatar signing failed");
            None
        }
    }
}
