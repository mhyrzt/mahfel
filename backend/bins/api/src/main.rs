mod api_docs_export;
mod cli;
mod http;
mod migrate;
mod seed;
mod serve;
mod settings;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let command = cli::command();
    if let cli::Command::ApiDocs {
        command: cli::ApiDocsCommand::Export { directory },
    } = command
    {
        return api_docs_export::run(&directory);
    }

    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;

    match command {
        cli::Command::Serve => serve::run(settings).await,
        cli::Command::Migrate => migrate::run(settings).await,
        cli::Command::Seed { path } => seed::run(settings, path).await,
        cli::Command::ApiDocs { .. } => unreachable!(),
    }
}
