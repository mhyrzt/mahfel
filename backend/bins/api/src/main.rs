mod cli;
mod http;
mod migrate;
mod serve;
mod settings;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;

    match cli::command().context("parse command")? {
        cli::Command::Serve => serve::run(settings).await,
        cli::Command::Migrate => migrate::run(settings).await,
    }
}
