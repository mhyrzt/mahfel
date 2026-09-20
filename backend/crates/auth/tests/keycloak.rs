use mahfel_auth::{TokenError, TokenVerifier, TokenVerifierConfig};
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor, wait::HttpWaitStrategy},
    runners::AsyncRunner,
};

/// Client used to obtain a real Keycloak access token.
///
/// The service account carries the `mahfel-api` audience mapper, so its token
/// is verifiable by the resource server exactly like a browser token.
const ADMIN_CLIENT_ID: &str = "mahfel-admin";
const ADMIN_CLIENT_SECRET: &str = "mahfel-admin-dev-secret";
/// Audience the API accepts.
const API_AUDIENCE: &str = "mahfel-api";

#[tokio::test]
async fn keycloak_tokens_verify_and_reject_tampering() -> Result<(), Box<dyn Error>> {
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

    // The API manages the realm with the client-credentials service account.
    let admin_token = client_credentials(&issuer, ADMIN_CLIENT_ID, ADMIN_CLIENT_SECRET).await?;
    let response = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/admin/realms/rastai/users"))
        .bearer_auth(&admin_token)
        .send()
        .await?;
    assert_eq!(
        response.status(),
        200,
        "service account must read realm users"
    );

    // The service account's token carries the `mahfel-api` audience, so it
    // verifies and normalizes exactly like a browser token would.
    let access_token = admin_token.clone();
    let verifier = TokenVerifier::new(TokenVerifierConfig::new(issuer.clone(), API_AUDIENCE));
    let identity = verifier.verify(&access_token).await?;
    assert!(
        !identity.subject.is_empty(),
        "the service-account token must carry a subject"
    );

    // A tampered signature is rejected.
    let mut tampered = access_token.clone().into_bytes();
    let last = tampered.len() - 1;
    tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(tampered)?;
    assert!(
        matches!(
            verifier.verify(&tampered).await,
            Err(TokenError::InvalidSignature)
        ),
        "a modified signature must not verify"
    );

    // A token issued for another audience is rejected.
    let other_audience =
        TokenVerifier::new(TokenVerifierConfig::new(issuer.clone(), "someone-else"));
    assert!(
        matches!(
            other_audience.verify(&access_token).await,
            Err(TokenError::WrongAudience)
        ),
        "a token for another audience must not verify"
    );

    // A token from another issuer is rejected outright.
    let other_issuer = TokenVerifier::new(TokenVerifierConfig::new(
        "http://127.0.0.1:1/realms/other",
        API_AUDIENCE,
    ));
    assert!(
        other_issuer.verify(&access_token).await.is_err(),
        "a token from another issuer must not verify"
    );

    Ok(())
}

async fn client_credentials(
    issuer: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String, Box<dyn Error>> {
    let token: serde_json::Value = reqwest::Client::new()
        .post(format!("{issuer}/protocol/openid-connect/token"))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access_token = token["access_token"]
        .as_str()
        .ok_or("token response did not contain an access token")?
        .to_owned();
    Ok(access_token)
}
