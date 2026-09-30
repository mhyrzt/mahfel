use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Config, Credentials, Region},
    types::{
        AbortIncompleteMultipartUpload, BucketLifecycleConfiguration, CorsConfiguration, CorsRule,
        ExpirationStatus, LifecycleExpiration, LifecycleRule, LifecycleRuleFilter,
    },
};
use base64::Engine;
use mahfel_application::ports::FileStorage;
use mahfel_storage::{S3FileStorage, S3Settings};
use sha2::{Digest, Sha256};
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use uuid::Uuid;

#[tokio::test]
async fn rustfs_supports_the_direct_transfer_contract() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping RustFS integration test");
        return Ok(());
    }
    let container = GenericImage::new("docker.arvancloud.ir/rustfs/rustfs", "1.0.0-rc.6")
        .with_exposed_port(9000.tcp())
        .with_wait_for(WaitFor::seconds(3))
        .with_env_var("RUSTFS_VOLUMES", "/data")
        .with_env_var("RUSTFS_ADDRESS", "0.0.0.0:9000")
        .with_env_var("RUSTFS_ACCESS_KEY", "mahfel-test")
        .with_env_var("RUSTFS_SECRET_KEY", "mahfel-test-secret")
        .start()
        .await?;
    let port = container.get_host_port_ipv4(9000).await?;
    let endpoint = format!("http://127.0.0.1:{port}");
    let settings = S3Settings {
        internal_endpoint: endpoint.clone(),
        public_endpoint: endpoint,
        region: "us-east-1".into(),
        bucket: format!("mahfel-test-{}", Uuid::new_v4().simple()),
        access_key: "mahfel-test".into(),
        secret_key: "mahfel-test-secret".into(),
        force_path_style: true,
    };
    exercise(settings).await
}

async fn exercise(settings: S3Settings) -> Result<(), Box<dyn Error>> {
    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(settings.region.clone()))
        .credentials_provider(Credentials::new(
            &settings.access_key,
            &settings.secret_key,
            None,
            None,
            "test",
        ))
        .endpoint_url(&settings.internal_endpoint)
        .force_path_style(true)
        .build();
    let client = Client::from_conf(config);
    let mut last_error = None;
    for _ in 0..20 {
        match client.create_bucket().bucket(&settings.bucket).send().await {
            Ok(_) => {
                last_error = None;
                break;
            }
            Err(error) => {
                last_error = Some(error);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    if let Some(error) = last_error {
        return Err(error.into());
    }
    client
        .put_bucket_cors()
        .bucket(&settings.bucket)
        .cors_configuration(
            CorsConfiguration::builder()
                .cors_rules(
                    CorsRule::builder()
                        .allowed_methods("PUT")
                        .allowed_methods("GET")
                        .allowed_origins("https://mahfel.localhost")
                        .allowed_headers("*")
                        .build()?,
                )
                .build()?,
        )
        .send()
        .await
        .map_err(|error| format!("configure bucket CORS: {error}"))?;
    client
        .put_bucket_lifecycle_configuration()
        .bucket(&settings.bucket)
        .lifecycle_configuration(
            BucketLifecycleConfiguration::builder()
                .rules(
                    LifecycleRule::builder()
                        .id("expire-abandoned-uploads")
                        .filter(LifecycleRuleFilter::builder().prefix("uploads/").build())
                        .status(ExpirationStatus::Enabled)
                        .expiration(LifecycleExpiration::builder().days(1).build())
                        .build()?,
                )
                .rules(
                    LifecycleRule::builder()
                        .id("abort-incomplete-recording-exports")
                        .filter(LifecycleRuleFilter::builder().prefix("recordings/").build())
                        .status(ExpirationStatus::Enabled)
                        .abort_incomplete_multipart_upload(
                            AbortIncompleteMultipartUpload::builder()
                                .days_after_initiation(1)
                                .build(),
                        )
                        .build()?,
                )
                .build()?,
        )
        .send()
        .await
        .map_err(|error| format!("configure bucket lifecycle: {error}"))?;
    let bucket = settings.bucket.clone();
    let storage = S3FileStorage::new(settings);
    storage.check_bucket().await?;

    let bytes = b"%PDF-1.7\n%%EOF".to_vec();
    let checksum = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&bytes));
    let seed_key = format!("uploads/{}", Uuid::new_v4());
    storage
        .put_seed_object(&seed_key, "application/pdf", &checksum, bytes.clone())
        .await?;
    let seeded = storage.read(&seed_key, 50 * 1024 * 1024).await?;
    assert_eq!(seeded.bytes, bytes);
    assert_eq!(seeded.content_type.as_deref(), Some("application/pdf"));
    storage.delete(&seed_key).await?;
    let key = format!("uploads/{}", Uuid::new_v4());
    let signed = storage
        .presign_upload(
            &key,
            "application/pdf",
            bytes.len() as u64,
            &checksum,
            Duration::from_secs(60),
        )
        .await?;
    let http = reqwest::Client::new();
    let mut put = http.put(&signed.url).body(bytes.clone());
    for (name, value) in signed.headers {
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        put = put.header(name, value);
    }
    let response = put.send().await?;
    assert!(
        response.status().is_success(),
        "signed PUT failed: {}",
        response.status()
    );
    let stored = storage.read(&key, 50 * 1024 * 1024).await?;
    assert_eq!(stored.bytes, bytes);
    assert_eq!(stored.content_type.as_deref(), Some("application/pdf"));
    let final_key = format!("files/{}", Uuid::new_v4());
    storage.promote(&key, &final_key).await?;
    assert_eq!(
        storage.read(&final_key, 50 * 1024 * 1024).await?.bytes,
        bytes
    );

    let download = storage
        .presign_download(&key, "deck.pdf", Duration::from_secs(60))
        .await?;
    let response = http.get(download.url).send().await?;
    assert!(
        response.status().is_success(),
        "signed GET failed: {}",
        response.status()
    );
    assert_eq!(response.bytes().await?.as_ref(), bytes);
    storage.delete(&key).await?;
    storage.delete(&final_key).await?;
    assert!(storage.read(&key, 50 * 1024 * 1024).await.is_err());
    let export_key = format!("recordings/test/exports/{}.mp4", Uuid::new_v4());
    let export_path = std::env::temp_dir().join(format!("mahfel-upload-{}.mp4", Uuid::new_v4()));
    let export_bytes = vec![42_u8; 17 * 1024 * 1024];
    tokio::fs::write(&export_path, &export_bytes).await?;
    mahfel_application::ports::RecordingStorage::upload_export(&storage, &export_key, &export_path)
        .await?;
    assert_eq!(
        storage.read(&export_key, 20 * 1024 * 1024).await?.bytes,
        export_bytes
    );
    let destination = export_path.with_extension("download");
    assert!(
        mahfel_application::ports::RecordingStorage::download(
            &storage,
            &export_key,
            &destination,
            1024
        )
        .await
        .is_err(),
        "download limit enforced"
    );
    mahfel_application::ports::RecordingStorage::download(
        &storage,
        &export_key,
        &destination,
        20 * 1024 * 1024,
    )
    .await?;
    assert_eq!(tokio::fs::read(&destination).await?, export_bytes);
    tokio::fs::remove_file(export_path).await?;
    tokio::fs::remove_file(destination).await?;
    storage.delete(&export_key).await?;
    client.delete_bucket().bucket(bucket).send().await?;
    Ok(())
}
