use crate::{
    policy::ServerCapabilityResolver,
    ports::{
        Capability, CapabilityContext, CapabilityResolver, FileStorage, Principal,
        RecordingPlaybackStore, RoomStore, SessionStore,
    },
    recordings::RecordingOperationError,
};
use mahfel_domain::SessionId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;
use uuid::Uuid;

const MANIFEST_MAX_BYTES: u64 = 8 * 1024 * 1024;
const PAGE_MAX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PAGES: usize = 4096;
pub(super) const INDEX_MAX_BYTES: u64 = 8 * 1024 * 1024;
pub(super) const MAX_MEDIA_SEGMENTS: usize = 10000;
const MEDIA_WINDOW_NS: i128 = 30_000_000_000;
const MAX_WINDOW_SEGMENTS: usize = 256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackMediaSegment {
    pub track_id: String,
    pub actor_id: String,
    pub source: String,
    pub audio: bool,
    pub start_ns: String,
    pub end_ns: String,
    pub object_key: String,
    pub sha256: String,
}

#[derive(Deserialize)]
struct PlaybackIndex {
    version: u32,
    clock: String,
    segments: Vec<PlaybackMediaSegment>,
}

async fn load_index<S: RecordingPlaybackStore + ?Sized, F: FileStorage + ?Sized>(
    store: &S,
    storage: &F,
    session: SessionId,
    recording: Uuid,
) -> Result<Vec<PlaybackMediaSegment>, RecordingOperationError> {
    let key = store
        .playback_index(session, recording)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let prefix = format!("recordings/{recording}/replay/");
    if !key.starts_with(&prefix) || key.contains("..") {
        return Err(RecordingOperationError::Unavailable);
    }
    let bytes = storage
        .read(&key, INDEX_MAX_BYTES)
        .await
        .map_err(|_| RecordingOperationError::Unavailable)?
        .bytes;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    if !key.ends_with(&format!("playback-{hash}.json")) {
        return Err(RecordingOperationError::Unavailable);
    }
    let index: PlaybackIndex =
        serde_json::from_slice(&bytes).map_err(|_| RecordingOperationError::Unavailable)?;
    if index.version != 1
        || index.clock != "session_relative_nanoseconds"
        || index.segments.len() > MAX_MEDIA_SEGMENTS
    {
        return Err(RecordingOperationError::Unavailable);
    }
    for segment in &index.segments {
        let start: i128 = segment
            .start_ns
            .parse()
            .map_err(|_| RecordingOperationError::Unavailable)?;
        let end: i128 = segment
            .end_ns
            .parse()
            .map_err(|_| RecordingOperationError::Unavailable)?;
        if start < 0
            || end < start
            || !segment.object_key.starts_with(&prefix)
            || segment.object_key.contains("..")
            || segment.sha256.len() != 64
            || !segment
                .object_key
                .ends_with(&format!("{}.mp4", segment.sha256))
        {
            return Err(RecordingOperationError::Unavailable);
        }
    }
    Ok(index.segments)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackDescriptor {
    pub manifest_key: String,
    pub urls: BTreeMap<String, String>,
    pub assets: BTreeMap<String, String>,
    pub media: Vec<PlaybackMediaSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackWindow {
    pub urls: BTreeMap<String, String>,
}

pub async fn playback_window<S: RecordingPlaybackStore + ?Sized, F: FileStorage + ?Sized>(
    store: &S,
    storage: &F,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
    start_ms: u64,
    expires: Duration,
) -> Result<PlaybackWindow, RecordingOperationError> {
    if !can_play(store, session, principal).await? {
        return Err(RecordingOperationError::NotFound);
    }
    let segments = load_index(store, storage, session, recording).await?;
    let start = i128::from(start_ms) * 1_000_000;
    let end = start.saturating_add(MEDIA_WINDOW_NS);
    let selected = segments
        .into_iter()
        .filter(|segment| {
            let from = segment.start_ns.parse::<i128>().unwrap_or(i128::MAX);
            let until = segment.end_ns.parse::<i128>().unwrap_or(i128::MIN);
            from < end && until > start
        })
        .collect::<Vec<_>>();
    if selected.len() > MAX_WINDOW_SEGMENTS {
        return Err(RecordingOperationError::Unavailable);
    }
    let mut urls = BTreeMap::new();
    for segment in selected {
        let url = storage
            .presign_playback(&segment.object_key, expires)
            .await
            .map_err(|_| RecordingOperationError::Unavailable)?;
        urls.insert(segment.object_key, url.url);
    }
    Ok(PlaybackWindow { urls })
}

async fn can_play<S: RecordingPlaybackStore + ?Sized>(
    store: &S,
    session: SessionId,
    principal: &Principal,
) -> Result<bool, RecordingOperationError> {
    if principal
        .roles
        .iter()
        .any(|role| role.eq_ignore_ascii_case("admin"))
    {
        return Ok(true);
    }
    Ok(store.can_play(session, principal.actor_id).await?)
}

fn page_name(key: &str) -> &str {
    key.rsplit('/').next().unwrap_or("replay.json")
}

/// Builds the authorized replay descriptor for a finished recording. Every
/// archived object the browser needs is returned as a short-lived presigned
/// URL; unauthorized callers never receive a URL.
pub async fn playback_descriptor<S: RecordingPlaybackStore + ?Sized, F: FileStorage + ?Sized>(
    store: &S,
    storage: &F,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
    expires: Duration,
) -> Result<PlaybackDescriptor, RecordingOperationError> {
    if !can_play(store, session, principal).await? {
        return Err(RecordingOperationError::NotFound);
    }
    let manifest_key = store
        .manifest(session, recording)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let media = load_index(store, storage, session, recording).await?;
    let manifest = storage
        .read(&manifest_key, MANIFEST_MAX_BYTES)
        .await
        .map_err(|_| RecordingOperationError::Unavailable)?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest.bytes)
        .map_err(|_| RecordingOperationError::Unavailable)?;
    if manifest.get("version").and_then(serde_json::Value::as_u64) != Some(1) {
        return Err(RecordingOperationError::Unavailable);
    }
    let mut page_keys = vec![manifest_key.clone()];
    let mut asset_keys = BTreeMap::new();
    if let Some(sections) = manifest
        .get("sections")
        .and_then(serde_json::Value::as_object)
    {
        for (section, pages) in sections {
            for page in pages.as_array().into_iter().flatten() {
                let key = page
                    .get("object_key")
                    .and_then(serde_json::Value::as_str)
                    .ok_or(RecordingOperationError::Unavailable)?
                    .to_owned();
                if section == "assets" {
                    let bytes = storage
                        .read(&key, PAGE_MAX_BYTES)
                        .await
                        .map_err(|_| RecordingOperationError::Unavailable)?;
                    let entries: Vec<serde_json::Value> = serde_json::from_slice(&bytes.bytes)
                        .map_err(|_| RecordingOperationError::Unavailable)?;
                    for entry in entries {
                        let file_id = entry
                            .get("file_id")
                            .and_then(serde_json::Value::as_str)
                            .ok_or(RecordingOperationError::Unavailable)?;
                        let archive_key = entry
                            .get("archive_key")
                            .and_then(serde_json::Value::as_str)
                            .ok_or(RecordingOperationError::Unavailable)?;
                        asset_keys.insert(file_id.to_owned(), archive_key.to_owned());
                    }
                }
                page_keys.push(key);
                if page_keys.len() > MAX_PAGES {
                    return Err(RecordingOperationError::Unavailable);
                }
            }
        }
    }
    let mut urls = BTreeMap::new();
    for key in &page_keys {
        let signed = storage
            .presign_download(key, page_name(key), expires)
            .await
            .map_err(|_| RecordingOperationError::Unavailable)?;
        urls.insert(key.clone(), signed.url);
    }
    let mut assets = BTreeMap::new();
    for (file_id, key) in asset_keys {
        if let Some(url) = urls.get(&key) {
            assets.insert(file_id, url.clone());
            continue;
        }
        let signed = storage
            .presign_download(&key, page_name(&key), expires)
            .await
            .map_err(|_| RecordingOperationError::Unavailable)?;
        assets.insert(file_id, signed.url);
    }
    Ok(PlaybackDescriptor {
        manifest_key,
        urls,
        assets,
        media,
    })
}

pub async fn delete_recording<
    S: RecordingPlaybackStore + ?Sized,
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
>(
    store: &S,
    rooms: &R,
    sessions: &T,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
) -> Result<(), RecordingOperationError> {
    let target = sessions
        .get(session)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let membership = rooms.get_member(target.room_id, principal.actor_id).await?;
    let capabilities = ServerCapabilityResolver.resolve(
        principal,
        CapabilityContext::in_room(membership.map(|member| member.role)),
    );
    if !capabilities.has(Capability::ManageSystem) && !can_play(store, session, principal).await? {
        return Err(RecordingOperationError::NotFound);
    }
    if !capabilities.has(Capability::ManageRecording) {
        return Err(RecordingOperationError::Forbidden);
    }
    Ok(store.delete(session, recording, principal.actor_id).await?)
}

pub async fn retry_playback<
    S: RecordingPlaybackStore + ?Sized,
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
>(
    store: &S,
    rooms: &R,
    sessions: &T,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
) -> Result<(), RecordingOperationError> {
    let target = sessions
        .get(session)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let membership = rooms.get_member(target.room_id, principal.actor_id).await?;
    let capabilities = ServerCapabilityResolver.resolve(
        principal,
        CapabilityContext::in_room(membership.map(|member| member.role)),
    );
    if !capabilities.has(Capability::ManageSystem) && !can_play(store, session, principal).await? {
        return Err(RecordingOperationError::NotFound);
    }
    if !capabilities.has(Capability::ManageRecording) {
        return Err(RecordingOperationError::Forbidden);
    }
    if !store.retry_processing(session, recording).await? {
        return Err(RecordingOperationError::Conflict);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApplicationError;
    use crate::ports::{
        BoxFuture, FileStorage, FileStorageError, PresignedFileRequest, RecordingPlaybackStore,
        StoreError, StoredFile,
    };
    use mahfel_domain::{ActorId, ActorKind};

    struct PlaybackStore {
        member: ActorId,
    }
    fn test_index(recording: Uuid) -> (String, Vec<u8>) {
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1, "clock": "session_relative_nanoseconds", "segments": [{
                "track_id": "track-1", "actor_id": Uuid::nil().to_string(),
                "source": "camera", "audio": false,
                "start_ns": "0", "end_ns": "10000000000",
                "object_key": format!("recordings/{recording}/replay/1/media/{}.mp4", "a".repeat(64)),
                "sha256": "a".repeat(64),
            }],
        }))
        .expect("index json");
        let hash = format!("{:x}", Sha256::digest(&bytes));
        (
            format!("recordings/{recording}/replay/1/playback-{hash}.json"),
            bytes,
        )
    }
    impl RecordingPlaybackStore for PlaybackStore {
        fn playback_index(
            &self,
            _session: SessionId,
            recording: Uuid,
        ) -> BoxFuture<'_, Result<Option<String>, StoreError>> {
            Box::pin(async move { Ok(Some(test_index(recording).0)) })
        }
        fn can_play(
            &self,
            _session: SessionId,
            actor: ActorId,
        ) -> BoxFuture<'_, Result<bool, StoreError>> {
            let member = self.member;
            Box::pin(async move { Ok(actor == member) })
        }
        fn manifest(
            &self,
            _session: SessionId,
            _recording: Uuid,
        ) -> BoxFuture<'_, Result<Option<String>, StoreError>> {
            Box::pin(async { Ok(Some("recordings/r/replay/1/manifest-abc.json".to_owned())) })
        }
        fn delete(
            &self,
            _session: SessionId,
            _recording: Uuid,
            _actor: ActorId,
        ) -> BoxFuture<'_, Result<(), StoreError>> {
            Box::pin(async { Ok(()) })
        }
    }

    struct ManifestStorage;
    impl FileStorage for ManifestStorage {
        fn presign_upload<'a>(
            &'a self,
            _key: &'a str,
            _content_type: &'a str,
            _size_bytes: u64,
            _checksum_sha256: &'a str,
            _expiry: Duration,
        ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
            Box::pin(async { Err(FileStorageError::Unavailable) })
        }
        fn read<'a>(
            &'a self,
            key: &'a str,
            _max_bytes: u64,
        ) -> BoxFuture<'a, Result<StoredFile, FileStorageError>> {
            let bytes = match key {
                "recordings/r/replay/1/manifest-abc.json" => {
                    serde_json::to_vec(&serde_json::json!({
                        "version": 1,
                        "sections": {
                            "events": [{"object_key": "recordings/r/replay/1/events/0-h.json", "sha256": "x", "entries": 0}],
                            "assets": [{"object_key": "recordings/r/replay/1/assets/0-h.json", "sha256": "x", "entries": 1}],
                        },
                    }))
                }
                "recordings/r/replay/1/assets/0-h.json" => serde_json::to_vec(
                    &serde_json::json!([{"file_id": "f1", "archive_key": "recordings/r/assets/f1"}]),
                ),
                _ if key.contains("/playback-") => {
                    let recording = key
                        .split('/')
                        .nth(1)
                        .and_then(|id| Uuid::parse_str(id).ok());
                    match recording {
                        Some(recording) if test_index(recording).0 == key => {
                            Ok(test_index(recording).1)
                        }
                        _ => return Box::pin(async { Err(FileStorageError::NotFound) }),
                    }
                }
                _ => return Box::pin(async { Err(FileStorageError::NotFound) }),
            };
            Box::pin(async move {
                Ok(StoredFile {
                    bytes: bytes.map_err(|_| FileStorageError::Unavailable)?,
                    content_type: None,
                })
            })
        }
        fn presign_download<'a>(
            &'a self,
            key: &'a str,
            _file_name: &'a str,
            _expiry: Duration,
        ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
            Box::pin(async move {
                Ok(PresignedFileRequest {
                    url: format!("https://storage.invalid/{key}"),
                    headers: Vec::new(),
                })
            })
        }
        fn delete<'a>(&'a self, _key: &'a str) -> BoxFuture<'a, Result<(), FileStorageError>> {
            Box::pin(async { Ok(()) })
        }
        fn promote<'a>(
            &'a self,
            _from: &'a str,
            _to: &'a str,
        ) -> BoxFuture<'a, Result<(), FileStorageError>> {
            Box::pin(async { Ok(()) })
        }
        fn check_bucket(&self) -> BoxFuture<'_, Result<(), FileStorageError>> {
            Box::pin(async { Ok(()) })
        }
    }

    fn principal(actor: ActorId) -> Principal {
        Principal {
            actor_id: actor,
            kind: ActorKind::User,
            display_name: None,
            avatar_url: None,
            email: None,
            email_verified: false,
            username: None,
            roles: Vec::new(),
        }
    }

    #[tokio::test]
    async fn authorized_member_receives_replay_urls() {
        let member = ActorId::from_uuid(Uuid::new_v4());
        let store = PlaybackStore { member };
        let recording = Uuid::new_v4();
        let session = SessionId::from_uuid(Uuid::new_v4());
        let descriptor = playback_descriptor(
            &store,
            &ManifestStorage,
            session,
            recording,
            &principal(member),
            Duration::from_secs(300),
        )
        .await
        .expect("member playback descriptor");
        assert_eq!(
            descriptor.manifest_key,
            "recordings/r/replay/1/manifest-abc.json"
        );
        assert!(
            descriptor
                .urls
                .contains_key("recordings/r/replay/1/manifest-abc.json")
        );
        assert!(
            descriptor
                .urls
                .contains_key("recordings/r/replay/1/events/0-h.json")
        );
        assert!(
            descriptor
                .urls
                .contains_key("recordings/r/replay/1/assets/0-h.json")
        );
        assert_eq!(
            descriptor.assets.get("f1").map(String::as_str),
            Some("https://storage.invalid/recordings/r/assets/f1")
        );
        assert_eq!(descriptor.media.len(), 1);
        let window = playback_window(
            &store,
            &ManifestStorage,
            session,
            recording,
            &principal(member),
            0,
            Duration::from_secs(300),
        )
        .await
        .expect("media window");
        assert_eq!(window.urls.len(), 1);
    }

    #[tokio::test]
    async fn unauthorized_caller_receives_no_urls() {
        let member = ActorId::from_uuid(Uuid::new_v4());
        let store = PlaybackStore { member };
        let error = playback_descriptor(
            &store,
            &ManifestStorage,
            SessionId::from_uuid(Uuid::new_v4()),
            Uuid::new_v4(),
            &principal(ActorId::from_uuid(Uuid::new_v4())),
            Duration::from_secs(300),
        )
        .await
        .expect_err("outsider must not obtain playback urls");
        assert_eq!(error.kind(), crate::ErrorKind::NotFound);
        let error = playback_window(
            &store,
            &ManifestStorage,
            SessionId::from_uuid(Uuid::new_v4()),
            Uuid::new_v4(),
            &principal(ActorId::from_uuid(Uuid::new_v4())),
            0,
            Duration::from_secs(300),
        )
        .await
        .expect_err("outsider must not obtain media urls");
        assert_eq!(error.kind(), crate::ErrorKind::NotFound);
    }
}
