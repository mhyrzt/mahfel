use anyhow::{Context, Result};
use std::{fs, path::Path};
use utoipa::OpenApi;

pub(crate) fn run(directory: &Path) -> Result<()> {
    let openapi = serde_json::to_string_pretty(&crate::http::routes::ApiDoc::openapi())
        .context("serialize OpenAPI document")?;
    let asyncapi = serde_json::to_string_pretty(&crate::http::ws::asyncapi::document()?)
        .context("serialize AsyncAPI document")?;
    fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    for (name, contents) in [("openapi.json", openapi), ("asyncapi.json", asyncapi)] {
        let path = directory.join(name);
        fs::write(&path, format!("{contents}\n"))
            .with_context(|| format!("write {}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
