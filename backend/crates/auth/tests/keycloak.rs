use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor, wait::HttpWaitStrategy},
    runners::AsyncRunner,
};

#[tokio::test]
async fn keycloak_discovery_and_client_credentials_use_a_dynamic_port() -> Result<(), Box<dyn Error>>
{
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping Keycloak integration test");
        return Ok(());
    }

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{port}/realms/rastai");
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

    let keycloak = GenericImage::new("quay.io/keycloak/keycloak", "26.7.4")
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
    assert_eq!(keycloak.get_host_port_ipv4(8080).await?, port);

    let discovery = reqwest::get(format!("{issuer}/.well-known/openid-configuration"))
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    assert_eq!(discovery["issuer"], issuer);
    assert!(discovery["authorization_endpoint"].is_string());
    assert!(discovery["token_endpoint"].is_string());
    assert!(discovery["jwks_uri"].is_string());

    // The API manages Keycloak with the client-credentials service account, not an admin user.
    let token: serde_json::Value = reqwest::Client::new()
        .post(format!("{issuer}/protocol/openid-connect/token"))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", "mahfel-admin"),
            ("client_secret", "mahfel-admin-dev-secret"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access_token = token["access_token"]
        .as_str()
        .expect("access token present");
    assert!(!access_token.is_empty());

    let users = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/admin/realms/rastai/users"))
        .bearer_auth(access_token)
        .send()
        .await?;
    assert_eq!(users.status(), 200, "service account must read realm users");
    Ok(())
}
