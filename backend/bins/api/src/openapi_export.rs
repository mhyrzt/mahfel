use anyhow::{Context, Result};
use std::{fs, path::Path};
use utoipa::OpenApi;

pub(crate) fn run(path: &Path) -> Result<()> {
    let document = serde_json::to_string_pretty(&crate::http::routes::ApiDoc::openapi())
        .context("serialize OpenAPI document")?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, format!("{document}\n"))
        .with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}
