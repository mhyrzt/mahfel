use mahfel_application::ports::{RoomStore, ServicePrincipalStore};
use mahfel_application::service_principals::{
    CreateServicePrincipal, CreateServicePrincipalError, RevokeServicePrincipal,
    RotateServicePrincipalKey, authenticate_service_principal, create_service_principal,
    revoke_service_principal, rotate_service_principal_key,
};
use mahfel_domain::{
    Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, ServicePrincipalStatus,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresRoomStore, PostgresServicePrincipalStore, connect,
    migrate,
};
use sqlx::PgPool;
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

fn test_pool_settings() -> PoolSettings {
    PoolSettings {
        min_connections: 1,
        max_connections: 2,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(30),
        max_lifetime: Duration::from_secs(60),
    }
}

#[tokio::test]
async fn service_principal_lifecycle_and_room_ownership() -> Result<(), Box<dyn Error>> {
    if let Ok(admin_url) = std::env::var("MAHFEL_TEST_DATABASE_URL") {
        return verify_disposable_database(&admin_url).await;
    }

    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!(
            "MAHFEL_TESTCONTAINERS is unset and MAHFEL_TEST_DATABASE_URL is unset; skipping PostgreSQL integration test"
        );
        return Ok(());
    }

    let postgres = GenericImage::new("postgres", "16-bookworm")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "postgres")
        .with_env_var("POSTGRES_USER", "mahfel")
        .with_env_var("POSTGRES_PASSWORD", "mahfel-test")
        .start()
        .await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let admin_url = format!("postgres://mahfel:mahfel-test@127.0.0.1:{port}/postgres");
    wait_until_ready(&admin_url).await?;
    verify_disposable_database(&admin_url).await
}

/// Wait until the container accepts an authenticated connection.
///
/// The `ready to accept connections` log line is emitted before the auth
/// backend can complete a handshake, so connecting immediately can be reset by
/// the server. Probe with a real pool instead of sleeping a fixed amount.
async fn wait_until_ready(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        match PgPool::connect(admin_url).await {
            Ok(pool) => {
                pool.close().await;
                return Ok(());
            }
            Err(error) if std::time::Instant::now() < deadline => {
                eprintln!("postgres not ready yet: {error}");
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

async fn verify_disposable_database(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let admin_pool = PgPool::connect(admin_url).await?;
    let database_name = format!("mahfel_test_{}", Uuid::new_v4().simple());
    let create_database = format!("CREATE DATABASE \"{database_name}\" TEMPLATE template0");
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_database))
        .execute(&admin_pool)
        .await?;

    let mut test_url = Url::parse(admin_url)?;
    test_url.set_path(&format!("/{database_name}"));
    let connection = connection_settings(&test_url)?;
    let pool = connect(&connection, test_pool_settings()).await?;
    migrate(&pool).await?;

    let verification = verify_service_principals(&pool).await;
    if !pool.is_closed() {
        pool.close().await;
    }

    let drop_database = format!("DROP DATABASE \"{database_name}\" WITH (FORCE)");
    let cleanup = sqlx::raw_sql(sqlx::AssertSqlSafe(drop_database))
        .execute(&admin_pool)
        .await;
    admin_pool.close().await;

    verification?;
    cleanup?;
    Ok(())
}

async fn verify_service_principals(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let now = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let principals = PostgresServicePrincipalStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());

    // Create a principal; the raw key is returned exactly once.
    let created = create_service_principal(
        &principals,
        CreateServicePrincipal {
            name: "lms-core".to_owned(),
            display_name: Some("LMS Core".to_owned()),
        },
    )
    .await?;
    assert_eq!(created.principal.status, ServicePrincipalStatus::Active);
    assert!(
        created.api_key.starts_with("mhf_"),
        "raw key carries the prefix"
    );

    // The principal is a service actor, not a user row.
    let kind = sqlx::query_scalar::<_, String>("SELECT kind FROM mahfel.actors WHERE id = $1")
        .bind(created.principal.id.as_uuid())
        .fetch_one(pool)
        .await?;
    assert_eq!(kind, "service", "a service principal is a service actor");

    // Authentication resolves the raw key to the active principal.
    let resolved = authenticate_service_principal(&principals, &created.api_key)
        .await?
        .expect("a fresh key must authenticate");
    assert_eq!(resolved.id, created.principal.id);
    assert_eq!(resolved.name.as_str(), "lms-core");

    // An unknown key resolves to nothing.
    assert!(
        authenticate_service_principal(&principals, "mhf_unknown")
            .await?
            .is_none()
    );

    // Names are unique; a duplicate is a typed conflict.
    let duplicate = create_service_principal(
        &principals,
        CreateServicePrincipal {
            name: "lms-core".to_owned(),
            display_name: None,
        },
    )
    .await;
    assert!(
        matches!(duplicate, Err(CreateServicePrincipalError::NameTaken)),
        "duplicate name must conflict"
    );

    // Rotation invalidates the previous key and issues a working new one.
    let rotated = rotate_service_principal_key(
        &principals,
        RotateServicePrincipalKey {
            id: created.principal.id,
        },
    )
    .await?;
    assert_ne!(rotated.api_key, created.api_key);
    assert!(
        authenticate_service_principal(&principals, &created.api_key)
            .await?
            .is_none(),
        "the old key must stop working after rotation"
    );
    let via_new = authenticate_service_principal(&principals, &rotated.api_key)
        .await?
        .expect("the new key must authenticate");
    assert_eq!(via_new.id, created.principal.id);

    // A service principal can own a room and become its manager, just like a
    // user.
    let room = Room::create(
        RoomTitle::new("LMS Room".to_owned())?,
        created.principal.id,
        RoomVisibility::InviteOnly,
        now,
    );
    let manager = RoomMember::new(room.id, created.principal.id, RoomRole::Manager, now);
    rooms.create_with_manager(&room, &manager).await?;
    assert_eq!(rooms.get(room.id).await?, Some(room.clone()));
    assert_eq!(
        rooms.get_member(room.id, created.principal.id).await?,
        Some(manager.clone())
    );
    assert_eq!(
        rooms.list_for_member(created.principal.id, 10, 0).await?,
        vec![room]
    );

    // Revocation rejects subsequent requests.
    revoke_service_principal(
        &principals,
        RevokeServicePrincipal {
            id: created.principal.id,
        },
    )
    .await?;
    assert!(
        authenticate_service_principal(&principals, &rotated.api_key)
            .await?
            .is_none(),
        "a revoked principal must no longer authenticate"
    );
    let stored = principals
        .get(created.principal.id)
        .await?
        .expect("the principal row is retained for audit");
    assert_eq!(stored.status, ServicePrincipalStatus::Revoked);

    // Rotating a revoked principal is refused.
    let rotate_revoked = rotate_service_principal_key(
        &principals,
        RotateServicePrincipalKey {
            id: created.principal.id,
        },
    )
    .await;
    assert!(rotate_revoked.is_err(), "a revoked principal cannot rotate");

    Ok(())
}

fn connection_settings(url: &Url) -> Result<ConnectionSettings, Box<dyn Error>> {
    Ok(ConnectionSettings {
        host: url
            .host_str()
            .ok_or("test database URL is missing a host")?
            .to_owned(),
        port: url.port().unwrap_or(5432),
        user: url.username().to_owned(),
        password: url.password().unwrap_or_default().to_owned(),
        name: url.path().trim_start_matches('/').to_owned(),
    })
}
