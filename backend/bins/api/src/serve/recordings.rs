use crate::http::ws::WsState;
use mahfel_application::{ports::RecordingStorage, recording_artifacts::FragmentReceipt};
use mahfel_domain::{RoomId, SessionId};
use mahfel_media_client::GrpcMediaGateway;
use mahfel_postgres::{CaptureIntent, PostgresRecordingStore};
use mahfel_rtc_protocol::media::ReconcileRecordingRequest;
use mahfel_storage::S3FileStorage;
use std::{sync::Arc, time::Duration};
use tokio::sync::watch;
use uuid::Uuid;

pub(crate) fn spawn(
    store: PostgresRecordingStore,
    storage: S3FileStorage,
    realtime: WsState,
    mut shutdown: watch::Receiver<bool>,
) {
    let archive_store = store.clone();
    let archive_storage = storage.clone();
    let mut archive_shutdown = shutdown.clone();
    tokio::spawn(async move {
        loop {
            if *archive_shutdown.borrow() {
                break;
            }
            if let Err(error) = archive_assets(&archive_store, &archive_storage).await {
                tracing::warn!(%error, "recording asset copy retry pending");
            }
            tokio::select! { _ = tokio::time::sleep(Duration::from_secs(1)) => {}, _ = archive_shutdown.changed() => break }
        }
    });
    tokio::spawn(async move {
        loop {
            if *shutdown.borrow() {
                break;
            }
            if let Err(error) = reconcile(&store, &storage, &realtime, None).await {
                tracing::warn!(%error, "recording reconciliation retry pending");
            }
            tokio::select! { _ = tokio::time::sleep(Duration::from_secs(1)) => {}, _ = shutdown.changed() => break }
        }
    });
}

pub(crate) async fn reconcile(
    store: &PostgresRecordingStore,
    storage: &S3FileStorage,
    realtime: &WsState,
    session: Option<SessionId>,
) -> anyhow::Result<()> {
    let mut records = std::collections::BTreeMap::<Uuid, Vec<CaptureIntent>>::new();
    for intent in store.capture_intents(session).await? {
        records.entry(intent.recording_id).or_default().push(intent);
    }
    let mut active = tokio::task::JoinSet::new();
    for (id, intents) in records {
        while active.len() >= 4 {
            if let Some(result) = active.join_next().await {
                result??;
            }
        }
        let store = store.clone();
        let storage = storage.clone();
        let realtime = realtime.clone();
        active.spawn(async move {
            for intent in intents {
                if let Err(error) = reconcile_one(&store, &storage, &realtime, &intent).await {
                    tracing::warn!(%error, recording_id = %intent.recording_id, generation = intent.generation, "recording media unavailable");
                    store.capture_unavailable(&intent).await?;
                }
            }
            store.finalize_sealed(id).await?;
            Ok::<_, anyhow::Error>(())
        });
    }
    while let Some(result) = active.join_next().await {
        result??;
    }
    Ok(())
}

async fn reconcile_one(
    store: &PostgresRecordingStore,
    storage: &S3FileStorage,
    realtime: &WsState,
    intent: &CaptureIntent,
) -> anyhow::Result<()> {
    let session = SessionId::from_uuid(intent.session_id);
    let room = RoomId::from_uuid(intent.room_id);
    let media: Arc<GrpcMediaGateway> = match &realtime.placement {
        Some(placement) if matches!(intent.status.as_str(), "starting" | "recording") => {
            placement.place(session, room).await?
        }
        _ => realtime.media_for(session, room).await?,
    };
    let desired = if intent.generation != intent.current_generation || intent.status == "paused" {
        "paused"
    } else if intent.status == "stopping" {
        "stopping"
    } else {
        "recording"
    };
    let media_room = if realtime.placement.is_some() {
        crate::http::ws::media_room_id(session)
    } else {
        room
    };
    let report = media
        .reconcile_recording(ReconcileRecordingRequest {
            recording_id: intent.recording_id.to_string(),
            room_id: media_room.to_string(),
            generation: u32::try_from(intent.generation)?,
            revision: u64::try_from(intent.revision)?,
            desired_state: desired.into(),
        })
        .await?;
    let boot = Uuid::parse_str(&report.boot_id)?;
    if store
        .capture_report(
            intent,
            &mahfel_postgres::CaptureObservation {
                boot,
                origin: report.unix_origin_ns,
                capturing: report.capturing,
                sealed: report.sealed,
                warning: (!report.warning.is_empty()).then_some(report.warning),
            },
        )
        .await?
        && intent.boot_id.is_none_or(|known| known == boot)
    {
        let prefix = format!(
            "recordings/{}/capture/{}/{boot}/receipts/",
            intent.recording_id, intent.generation
        );
        import_receipts(
            store,
            storage,
            &prefix,
            intent.recording_id,
            intent.generation,
            boot,
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn import_receipts(
    store: &PostgresRecordingStore,
    storage: &S3FileStorage,
    prefix: &str,
    recording: Uuid,
    generation: i32,
    boot: Uuid,
) -> anyhow::Result<()> {
    let mut continuation = None;
    loop {
        let page = storage.page(prefix, continuation).await?;
        for key in page.keys {
            let receipt: FragmentReceipt =
                serde_json::from_slice(&storage.read(&key, 8192).await?)?;
            if receipt.recording_id == recording
                && receipt.generation == u32::try_from(generation)?
                && receipt.boot_id == boot
            {
                store.import_fragment(&receipt).await?;
            }
        }
        continuation = page.continuation;
        if continuation.is_none() {
            break;
        }
    }
    Ok(())
}

async fn archive_assets(
    store: &PostgresRecordingStore,
    storage: &S3FileStorage,
) -> anyhow::Result<()> {
    for asset in store.pending_asset_copies().await? {
        if let Err(error) = archive_asset(store, storage, &asset).await {
            tracing::warn!(%error, recording_id = %asset.recording_id, file_id = %asset.file_id, "recording asset copy remains pinned");
        }
    }
    Ok(())
}

async fn archive_asset(
    store: &PostgresRecordingStore,
    storage: &S3FileStorage,
    asset: &mahfel_postgres::RecordingAssetCopy,
) -> anyhow::Result<()> {
    storage.copy(&asset.source_key, &asset.archive_key).await?;
    // Verify private archived bytes before releasing the source pin.
    let bytes = storage
        .read(&asset.archive_key, usize::try_from(asset.size_bytes)?)
        .await?;
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let (checksum, length) = tokio::task::spawn_blocking(move || {
        (
            base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&bytes)),
            bytes.len(),
        )
    })
    .await?;
    anyhow::ensure!(
        checksum == asset.checksum_sha_256 && length == usize::try_from(asset.size_bytes)?,
        "archived asset checksum mismatch"
    );
    store
        .accept_asset_copy(asset.recording_id, asset.file_id)
        .await?;
    Ok(())
}
