use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use base64::Engine;
use mahfel_application::{
    files::{CompleteUpload, complete_upload, verify_file_content},
    ports::{FileStore, RoomStore, StoredFile},
};
use mahfel_domain::{
    ActorId, FileContext, FileId, FileObject, FileStatus, MAX_FILE_BYTES, RoomId, RoomStatus,
    SessionId,
};
use mahfel_postgres::{PostgresFileStore, PostgresRoomStore};
use mahfel_storage::S3FileStorage;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::seed::fixture::{Fixture, parse_actor_id};

const MAX_SEED_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AssetSpec {
    id: String,
    owner_id: String,
    context_kind: AssetContext,
    context_id: String,
    path: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AssetContext {
    Avatar,
    Room,
    Session,
}

pub(super) struct SeedAsset {
    id: FileId,
    owner: ActorId,
    context: FileContext,
    path: PathBuf,
    file_name: String,
    mime: &'static str,
    prepared: Option<(FileObject, Vec<u8>)>,
}

pub(super) fn validate_assets(
    specs: Vec<AssetSpec>,
    base_dir: Option<&Path>,
) -> Result<Vec<SeedAsset>> {
    if specs.is_empty() {
        anyhow::bail!("files seed must contain at least one file");
    }
    let mut ids = HashSet::new();
    specs
        .into_iter()
        .map(|spec| {
            let id =
                FileId::from_uuid(uuid::Uuid::parse_str(&spec.id).context("validate file id")?);
            if !ids.insert(id) {
                anyhow::bail!("duplicate file id {id}");
            }
            let owner = parse_actor_id(&spec.owner_id).context("validate file owner id")?;
            let context_id =
                uuid::Uuid::parse_str(&spec.context_id).context("validate file context id")?;
            let context = match spec.context_kind {
                AssetContext::Avatar => FileContext::Avatar(ActorId::from_uuid(context_id)),
                AssetContext::Room => FileContext::Room(RoomId::from_uuid(context_id)),
                AssetContext::Session => FileContext::Session(SessionId::from_uuid(context_id)),
            };
            if let FileContext::Avatar(actor) = context
                && actor != owner
            {
                anyhow::bail!("avatar owner must match context actor");
            }
            let path = if spec.path.is_absolute() {
                spec.path
            } else {
                base_dir
                    .context("relative file path requires an explicit TOML seed path")?
                    .join(spec.path)
            };
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .context("file path has no UTF-8 file name")?
                .to_owned();
            let mime = match path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("png") => "image/png",
                Some("jpg" | "jpeg") => "image/jpeg",
                Some("pdf") if !matches!(context, FileContext::Avatar(_)) => "application/pdf",
                _ => anyhow::bail!("unsupported seed file extension: {}", path.display()),
            };
            Ok(SeedAsset {
                id,
                owner,
                context,
                path,
                file_name,
                mime,
                prepared: None,
            })
        })
        .collect()
}

pub(super) async fn preflight(seeds: &mut [(String, Fixture)]) -> Result<()> {
    let mut total = 0_u64;
    for (name, fixture) in seeds {
        let Fixture::Files(assets) = fixture else {
            continue;
        };
        for asset in assets {
            let metadata = tokio::fs::metadata(&asset.path)
                .await
                .with_context(|| format!("read seed file {} in {name}", asset.path.display()))?;
            if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FILE_BYTES {
                anyhow::bail!("seed file size is invalid: {}", asset.path.display());
            }
            total = total
                .checked_add(metadata.len())
                .context("seed asset size overflow")?;
            if total > MAX_SEED_BYTES {
                anyhow::bail!("seed assets exceed {} bytes", MAX_SEED_BYTES);
            }
            let bytes = tokio::fs::read(&asset.path)
                .await
                .with_context(|| format!("read seed file {} in {name}", asset.path.display()))?;
            if bytes.len() as u64 != metadata.len() {
                anyhow::bail!("seed file changed while reading: {}", asset.path.display());
            }
            let checksum = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&bytes));
            let mut file = FileObject::pending(
                asset.owner,
                asset.context,
                asset.file_name.clone(),
                asset.mime.to_owned(),
                metadata.len(),
                checksum,
                time::OffsetDateTime::now_utc(),
            )
            .with_context(|| format!("validate seed file {}", asset.path.display()))?;
            file.id = asset.id;
            file.object_key = format!("uploads/{}", asset.id);
            if verify_file_content(
                &file,
                &StoredFile {
                    bytes: bytes.clone(),
                    content_type: Some(asset.mime.to_owned()),
                },
            )
            .is_none()
            {
                anyhow::bail!("seed file content is invalid: {}", asset.path.display());
            }
            asset.prepared = Some((file, bytes));
        }
    }
    Ok(())
}

pub(super) async fn apply_assets(
    files: &PostgresFileStore,
    rooms: &PostgresRoomStore,
    storage: &S3FileStorage,
    assets: &[SeedAsset],
    seed_name: &str,
) -> Result<()> {
    for asset in assets {
        let (file, bytes) = asset
            .prepared
            .as_ref()
            .context("seed asset was not prepared")?;
        ensure_access(files, rooms, file)
            .await
            .with_context(|| format!("authorize file {} from seed {seed_name}", asset.id))?;
        if let Some(existing) = files.get(file.id).await? {
            if existing.owner != file.owner
                || existing.context != file.context
                || existing.checksum_sha256 != file.checksum_sha256
                || existing.size_bytes != file.size_bytes
                || existing.declared_mime != file.declared_mime
            {
                anyhow::bail!("seed file {} conflicts with existing metadata", file.id);
            }
            if existing.status == FileStatus::Ready {
                continue;
            }
            if !matches!(
                existing.status,
                FileStatus::Pending | FileStatus::Quarantined
            ) {
                anyhow::bail!(
                    "seed file {} cannot resume from status {}",
                    file.id,
                    existing.status.as_str()
                );
            }
        } else {
            files
                .insert(file)
                .await
                .with_context(|| format!("insert file {} from seed {seed_name}", file.id))?;
        }
        storage
            .put_seed_object(
                &file.object_key,
                &file.declared_mime,
                &file.checksum_sha256,
                bytes.clone(),
            )
            .await
            .with_context(|| format!("upload file {} from seed {seed_name}", file.id))?;
        complete_upload(
            files,
            storage,
            CompleteUpload {
                actor: file.owner,
                id: file.id,
            },
        )
        .await
        .with_context(|| format!("verify file {} from seed {seed_name}", file.id))?;
        tracing::info!(file_id = %file.id, seed = %seed_name, "seed file applied");
    }
    Ok(())
}

async fn ensure_access(
    files: &PostgresFileStore,
    rooms: &PostgresRoomStore,
    file: &FileObject,
) -> Result<()> {
    if matches!(file.context, FileContext::Avatar(_)) {
        return Ok(());
    }
    let room_id = files
        .context_room(file.context)
        .await?
        .context("file context does not exist")?;
    let room = rooms
        .get(room_id)
        .await?
        .context("file room does not exist")?;
    if room.status != RoomStatus::Active || rooms.get_member(room_id, file.owner).await?.is_none() {
        anyhow::bail!("file owner is not an active room member");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AssetContext, AssetSpec, validate_assets};
    use std::path::{Path, PathBuf};

    fn spec(path: PathBuf) -> AssetSpec {
        AssetSpec {
            id: "00000000-0000-4000-8000-000000000301".into(),
            owner_id: "00000000-0000-4000-8000-000000000004".into(),
            context_kind: AssetContext::Avatar,
            context_id: "00000000-0000-4000-8000-000000000004".into(),
            path,
        }
    }

    #[test]
    fn resolves_relative_and_absolute_asset_paths() {
        let base = Path::new("/tmp/mahfel-seeds");
        let relative = validate_assets(vec![spec("assets/avatar.png".into())], Some(base))
            .expect("relative path");
        assert_eq!(relative[0].path, base.join("assets/avatar.png"));
        let absolute =
            validate_assets(vec![spec("/tmp/avatar.png".into())], None).expect("absolute path");
        assert_eq!(absolute[0].path, Path::new("/tmp/avatar.png"));
        assert!(validate_assets(vec![spec("avatar.png".into())], None).is_err());
    }

    #[tokio::test]
    async fn rejects_invalid_file_content_before_writes() {
        let path = std::env::temp_dir().join(format!("mahfel-seed-{}.png", uuid::Uuid::new_v4()));
        tokio::fs::write(&path, b"not a PNG")
            .await
            .expect("write file");
        let assets = validate_assets(vec![spec(path.clone())], None).expect("fixture");
        let mut seeds = vec![("files.toml".to_owned(), super::Fixture::Files(assets))];
        assert!(super::preflight(&mut seeds).await.is_err());
        tokio::fs::remove_file(path).await.expect("remove file");
    }
}
