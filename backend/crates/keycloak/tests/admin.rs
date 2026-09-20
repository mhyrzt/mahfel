use mahfel_application::ports::{AttributeError, UserAttributeWriter};
use mahfel_keycloak::{KeycloakAdmin, KeycloakAdminSettings};
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor, wait::HttpWaitStrategy},
    runners::AsyncRunner,
};

const REALM: &str = "rastai";
const ADMIN_CLIENT_ID: &str = "mahfel-admin";
const ADMIN_CLIENT_SECRET: &str = "mahfel-admin-dev-secret";
const SERVICE_ACCOUNT: &str = "service-account-mahfel-admin";

struct Keycloak {
    base_url: String,
}

async fn start_keycloak() -> Result<Option<Keycloak>, Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping Keycloak admin integration test");
        return Ok(None);
    }

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let hostname = format!("http://127.0.0.1:{port}");
    let realm = include_bytes!("../../../../deploy/keycloak/rastai-realm.json").to_vec();
    let command = vec![
        "start-dev".to_owned(),
        "--import-realm".to_owned(),
        "--db=dev-mem".to_owned(),
        "--http-enabled=true".to_owned(),
        format!("--hostname={hostname}"),
        "--hostname-backchannel-dynamic=true".to_owned(),
        "--proxy-headers=xforwarded".to_owned(),
    ];

    let container = GenericImage::new("quay.io/keycloak/keycloak", "26.7.4")
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/realms/rastai/.well-known/openid-configuration")
                .with_port(8080.tcp())
                .with_expected_status_code(200u16)
                .with_poll_interval(Duration::from_secs(2)),
        ))
        .with_exposed_port(8080.tcp())
        .with_mapped_port(port, 8080.tcp())
        .with_copy_to("/opt/keycloak/data/import/rastai-realm.json", realm)
        .with_startup_timeout(Duration::from_secs(240))
        .with_cmd(command)
        .start()
        .await?;
    assert_eq!(container.get_host_port_ipv4(8080).await?, port);
    // Keep the container alive for the test duration.
    std::mem::forget(container);

    Ok(Some(Keycloak {
        base_url: format!("http://127.0.0.1:{port}"),
    }))
}

fn settings(keycloak: &Keycloak, secret: &str) -> KeycloakAdminSettings {
    KeycloakAdminSettings {
        base_url: keycloak.base_url.clone(),
        realm: REALM.to_owned(),
        client_id: ADMIN_CLIENT_ID.to_owned(),
        client_secret: secret.to_owned(),
        http_timeout: Duration::from_secs(5),
    }
}

async fn service_account_id(keycloak: &Keycloak) -> Result<String, Box<dyn Error>> {
    let token: serde_json::Value = reqwest::Client::new()
        .post(format!(
            "{}/realms/{REALM}/protocol/openid-connect/token",
            keycloak.base_url
        ))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", ADMIN_CLIENT_ID),
            ("client_secret", ADMIN_CLIENT_SECRET),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access_token = token["access_token"]
        .as_str()
        .ok_or("token response missing access_token")?;

    let users: serde_json::Value = reqwest::Client::new()
        .get(format!(
            "{}/admin/realms/{REALM}/users?username={SERVICE_ACCOUNT}",
            keycloak.base_url
        ))
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    users[0]["id"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "service account user not found".into())
}

#[tokio::test]
async fn sets_and_replaces_a_user_attribute_idempotently() -> Result<(), Box<dyn Error>> {
    let Some(keycloak) = start_keycloak().await? else {
        return Ok(());
    };
    let admin = KeycloakAdmin::new(settings(&keycloak, ADMIN_CLIENT_SECRET));
    let user_id = service_account_id(&keycloak).await?;

    UserAttributeWriter::set_user_attribute(
        &admin,
        &user_id,
        "mahfel_user_id",
        "11111111-1111-4111-8111-111111111111",
    )
    .await?;

    assert_eq!(
        stored_attribute(&keycloak, &user_id).await?,
        Some("11111111-1111-4111-8111-111111111111".to_owned())
    );

    // Re-running the link replaces the value without error.
    UserAttributeWriter::set_user_attribute(
        &admin,
        &user_id,
        "mahfel_user_id",
        "22222222-2222-4222-8222-222222222222",
    )
    .await?;
    assert_eq!(
        stored_attribute(&keycloak, &user_id).await?,
        Some("22222222-2222-4222-8222-222222222222".to_owned())
    );

    Ok(())
}

#[tokio::test]
async fn a_rejected_service_account_is_classified_as_unexpected() -> Result<(), Box<dyn Error>> {
    let Some(keycloak) = start_keycloak().await? else {
        return Ok(());
    };
    let admin = KeycloakAdmin::new(settings(&keycloak, "wrong-secret"));

    let error = UserAttributeWriter::set_user_attribute(
        &admin,
        "00000000-0000-0000-0000-000000000000",
        "mahfel_user_id",
        "11111111-1111-4111-8111-111111111111",
    )
    .await
    .expect_err("a rejected service account must fail");

    assert!(
        matches!(error, AttributeError::Unexpected(_)),
        "a rejected service account is a configuration fault, got {error:?}"
    );
    Ok(())
}

async fn stored_attribute(
    keycloak: &Keycloak,
    user_id: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let token: serde_json::Value = reqwest::Client::new()
        .post(format!(
            "{}/realms/{REALM}/protocol/openid-connect/token",
            keycloak.base_url
        ))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", ADMIN_CLIENT_ID),
            ("client_secret", ADMIN_CLIENT_SECRET),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access_token = token["access_token"]
        .as_str()
        .ok_or("token response missing access_token")?;

    let user: serde_json::Value = reqwest::Client::new()
        .get(format!(
            "{}/admin/realms/{REALM}/users/{user_id}",
            keycloak.base_url
        ))
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(user["attributes"]["mahfel_user_id"][0]
        .as_str()
        .map(str::to_owned))
}
