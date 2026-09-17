use std::error::Error;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

#[tokio::test]
async fn dex_discovery_uses_a_dynamic_testcontainers_port() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping Dex integration test");
        return Ok(());
    }

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{port}/dex");
    let config = include_str!("../../../../deploy/dex/config.yaml")
        .replace("https://auth.mahfel.localhost/dex", &issuer)
        .into_bytes();
    let dex = GenericImage::new("ghcr.io/dexidp/dex", "v2.41.1")
        .with_wait_for(WaitFor::http(
            testcontainers::core::wait::HttpWaitStrategy::new(
                "/dex/.well-known/openid-configuration",
            )
            .with_port(5556.tcp())
            .with_expected_status_code(200u16),
        ))
        .with_exposed_port(5556.tcp())
        .with_mapped_port(port, 5556.tcp())
        .with_copy_to("/etc/dex/config.yaml", config)
        .with_cmd(["dex", "serve", "/etc/dex/config.yaml"])
        .start()
        .await?;
    assert_eq!(dex.get_host_port_ipv4(5556).await?, port);
    let discovery = reqwest::get(format!("{issuer}/.well-known/openid-configuration"))
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    assert_eq!(discovery["issuer"], issuer);
    assert!(discovery["authorization_endpoint"].is_string());
    assert!(discovery["token_endpoint"].is_string());
    assert!(discovery["jwks_uri"].is_string());
    Ok(())
}
