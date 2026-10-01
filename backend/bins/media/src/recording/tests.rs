use super::*;
use mahfel_application::ports::{BoxFuture, FileStorageError, ObjectPage};
use std::sync::Mutex as SyncMutex;

#[derive(Default)]
struct Storage {
    objects: SyncMutex<HashMap<String, Vec<u8>>>,
    fail: AtomicBool,
}
impl RecordingStorage for Storage {
    fn put<'a>(
        &'a self,
        key: &'a str,
        bytes: Vec<u8>,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            if self.fail.load(Ordering::Acquire) {
                return Err(FileStorageError::Unavailable);
            }
            self.objects
                .lock()
                .expect("objects")
                .insert(key.into(), bytes);
            Ok(())
        })
    }
    fn put_fragment<'a>(
        &'a self,
        key: &'a str,
        path: &'a std::path::Path,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            self.put(
                key,
                tokio::fs::read(path)
                    .await
                    .map_err(|_| FileStorageError::Unavailable)?,
            )
            .await
        })
    }
    fn read<'a>(
        &'a self,
        key: &'a str,
        _maximum: usize,
    ) -> BoxFuture<'a, Result<Vec<u8>, FileStorageError>> {
        Box::pin(async move {
            self.objects
                .lock()
                .expect("objects")
                .get(key)
                .cloned()
                .ok_or(FileStorageError::NotFound)
        })
    }
    fn page<'a>(
        &'a self,
        _prefix: &'a str,
        _continuation: Option<String>,
    ) -> BoxFuture<'a, Result<ObjectPage, FileStorageError>> {
        Box::pin(async {
            Ok(ObjectPage {
                keys: vec![],
                continuation: None,
            })
        })
    }
    fn copy<'a>(
        &'a self,
        _source: &'a str,
        _destination: &'a str,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async { Ok(()) })
    }
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            self.objects.lock().expect("objects").remove(key);
            Ok(())
        })
    }
}

#[tokio::test]
async fn commands_are_fenced_and_sealed_generations_cannot_reactivate() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("free port");
    let port = listener.local_addr().expect("port").port();
    drop(listener);
    let media = Arc::new(
        MediaRuntime::start(1, port, port, "127.0.0.1", "127.0.0.1")
            .await
            .expect("media"),
    );
    let storage = Arc::new(Storage::default());
    let settings = RecordingSettings {
        spool_directory: std::env::temp_dir()
            .join(format!("mahfel-coordinator-{}", Uuid::new_v4())),
        recording_spool_bytes: 32,
        node_spool_bytes: 64,
        ..Default::default()
    };
    let records = Recordings::new(
        media.clone(),
        storage.clone(),
        settings.clone(),
        &prometheus::Registry::new(),
    )
    .expect("coordinator");
    let mut request = ReconcileRecordingRequest {
        recording_id: Uuid::new_v4().to_string(),
        room_id: Uuid::new_v4().to_string(),
        generation: 1,
        revision: 1,
        desired_state: "recording".into(),
    };
    storage.fail.store(true, Ordering::Release);
    assert!(
        records.prepare(&request.room_id).await.is_err(),
        "storage outage must block preflight"
    );
    assert!(
        !media
            .has_room(mahfel_domain::RoomId::from_uuid(
                Uuid::parse_str(&request.room_id).expect("room id")
            ))
            .await,
        "failed preflight must not allocate a media room"
    );
    storage.fail.store(false, Ordering::Release);
    records
        .prepare(&request.room_id)
        .await
        .expect("capture dependencies ready");
    assert!(
        storage.objects.lock().expect("objects").is_empty(),
        "preflight object removed"
    );
    records.reconcile(request.clone()).await.expect("start");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        records
            .reconcile(request.clone())
            .await
            .expect("started")
            .capturing
    );
    request.revision = 2;
    request.desired_state = "paused".into();
    let paused = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let report = records.reconcile(request.clone()).await.expect("pause");
            if report.sealed {
                break report;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("sealed promptly");
    assert!(!paused.capturing);
    request.desired_state = "recording".into();
    request.revision = 3;
    assert!(records.reconcile(request.clone()).await.is_err());
    request.generation = 2;
    records
        .reconcile(request.clone())
        .await
        .expect("resume fresh generation");
    let pending = settings
        .spool_directory
        .join(&request.recording_id)
        .join("1")
        .join("pending.bin");
    tokio::fs::create_dir_all(pending.parent().expect("pending directory"))
        .await
        .expect("old generation directory");
    tokio::fs::write(&pending, [0_u8; 32])
        .await
        .expect("pending bytes");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let report = records.reconcile(request.clone()).await.expect("pressure");
            if report.warning == "spool_exhausted" {
                assert!(!report.capturing);
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("prior generation bytes enforce recording quota");
    tokio::fs::remove_file(pending)
        .await
        .expect("uploaded pending bytes");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let report = records.reconcile(request.clone()).await.expect("recovery");
            if report.capturing && report.warning.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("capture recovers after pressure clears");
    request.revision = 2;
    assert!(records.reconcile(request.clone()).await.is_err());
    request.revision = 3;
    request.room_id = Uuid::new_v4().to_string();
    assert!(records.reconcile(request).await.is_err());
    records.drain().await;
    let _ = tokio::fs::remove_dir_all(settings.spool_directory).await;
}

#[tokio::test]
async fn spool_receipts_survive_upload_failure_and_recover_after_restart() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("free port");
    let port = listener.local_addr().expect("port").port();
    drop(listener);
    let media = Arc::new(
        MediaRuntime::start(1, port, port, "127.0.0.1", "127.0.0.1")
            .await
            .expect("media"),
    );
    let storage = Arc::new(Storage::default());
    let root = std::env::temp_dir().join(format!("mahfel-spool-{}", Uuid::new_v4()));
    let settings = RecordingSettings {
        spool_directory: root.clone(),
        ..Default::default()
    };
    let records = Recordings::new(
        Arc::clone(&media),
        storage.clone(),
        settings.clone(),
        &prometheus::Registry::new(),
    )
    .expect("coordinator");
    let directory = root.join("track").join("instance");
    tokio::fs::create_dir_all(&directory)
        .await
        .expect("spool directory");
    let fragment = directory.join("000000.webm");
    let bytes = b"original-encoded-fixture".to_vec();
    let receipt = FragmentReceipt {
        recording_id: Uuid::new_v4(),
        generation: 1,
        boot_id: Uuid::new_v4(),
        producer_id: Uuid::new_v4().to_string(),
        actor_id: Uuid::new_v4(),
        source: "microphone".into(),
        codec: "audio/opus".into(),
        container: "webm".into(),
        unix_origin_ns: 1,
        start_ns: 0,
        end_ns: 5_000_000_000,
        object_key: "recordings/fixture.webm".into(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        bytes: bytes.len() as u64,
    };
    tokio::fs::write(&fragment, &bytes).await.expect("fragment");
    tokio::fs::write(
        fragment.with_extension("receipt"),
        serde_json::to_vec(&receipt).expect("receipt"),
    )
    .await
    .expect("journal");
    storage.fail.store(true, Ordering::Release);
    assert!(records.upload_pending().await.is_err());
    assert!(fragment.exists());
    assert!(fragment.with_extension("receipt").exists());
    drop(records);
    storage.fail.store(false, Ordering::Release);
    let restarted = Recordings::new(
        media,
        storage.clone(),
        settings,
        &prometheus::Registry::new(),
    )
    .expect("restarted");
    restarted.upload_pending().await.expect("upload recovery");
    assert!(!fragment.exists());
    assert!(!fragment.with_extension("receipt").exists());
    assert_eq!(
        storage
            .read(&receipt.object_key, 1024)
            .await
            .expect("durable original"),
        bytes
    );
    assert_eq!(
        storage.objects.lock().expect("objects").len(),
        2,
        "fragment and durable receipt both uploaded"
    );
    let _ = tokio::fs::remove_dir_all(root).await;
}
