mod cli;
mod http;
mod migrate;
mod openapi_export;
mod seed;
mod serve;
mod settings;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let command = cli::command().context("parse command")?;
    if let cli::Command::OpenApiExport { path } = command {
        return openapi_export::run(&path);
    }

    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;

    match command {
        cli::Command::Serve => serve::run(settings).await,
        cli::Command::Migrate => migrate::run(settings).await,
        cli::Command::Seed { path } => seed::run(settings, path).await,
        cli::Command::OpenApiExport { .. } => unreachable!(),
    }
}
