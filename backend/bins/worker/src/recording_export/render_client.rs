use anyhow::{Context, Result, ensure};
use reqwest::{Client, redirect::Policy};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct RenderUnavailable;
impl std::fmt::Display for RenderUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("rendering service unavailable")
    }
}
impl std::error::Error for RenderUnavailable {}

pub(super) struct RenderClient {
    client: Client,
    url: String,
    key: String,
    previous: Option<Value>,
    png: Vec<u8>,
}
impl RenderClient {
    pub fn new() -> Result<Self> {
        let url = std::env::var("MAHFEL_WORKER__EXPORTS__RENDERER_URL")
            .context("renderer URL missing")?;
        let key = std::env::var("MAHFEL_WORKER__EXPORTS__RENDERER_API_KEY")
            .context("renderer API key missing")?;
        ensure!(!key.is_empty(), "renderer API key missing");
        let parsed = reqwest::Url::parse(&url).context("invalid renderer URL")?;
        ensure!(
            matches!(parsed.scheme(), "http" | "https")
                && parsed.username().is_empty()
                && parsed.password().is_none(),
            "invalid renderer URL"
        );
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(35))
                .redirect(Policy::none())
                .build()?,
            url,
            key,
            previous: None,
            png: Vec::new(),
        })
    }
    pub async fn render(&mut self, request: Value) -> Result<Vec<u8>> {
        if self.previous.as_ref() == Some(&request) {
            return Ok(self.png.clone());
        }
        let body = serde_json::to_vec(&request)?;
        ensure!(
            body.len() <= 8 * 1024 * 1024,
            "render request resource limit"
        );
        let mut response = self
            .client
            .post(&self.url)
            .bearer_auth(&self.key)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| RenderUnavailable)?;
        let status = response.status();
        if status.is_server_error() || status.as_u16() == 429 {
            return Err(RenderUnavailable.into());
        }
        ensure!(
            status.is_success(),
            "rendering request rejected: HTTP {}",
            status.as_u16()
        );
        ensure!(
            response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                == Some("image/png"),
            "invalid renderer content type"
        );
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| RenderUnavailable)? {
            ensure!(
                bytes.len() + chunk.len() <= 16 * 1024 * 1024,
                "render response resource limit"
            );
            bytes.extend_from_slice(&chunk);
        }
        ensure!(
            bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "invalid renderer PNG"
        );
        self.previous = Some(request);
        self.png = bytes.clone();
        Ok(bytes)
    }
}
