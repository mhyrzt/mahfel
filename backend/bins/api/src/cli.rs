use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mahfel-api")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    Serve,
    Migrate,
    ApiDocs {
        #[command(subcommand)]
        command: ApiDocsCommand,
    },
    Seed {
        path: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum ApiDocsCommand {
    Export {
        #[arg(default_value = "specs")]
        directory: PathBuf,
    },
}

pub(crate) fn command() -> Command {
    Cli::parse().command.unwrap_or(Command::Serve)
}

#[cfg(test)]
mod tests {
    use super::{ApiDocsCommand, Cli, Command};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn defaults_api_docs_export_to_the_repository_directory() {
        let cli =
            Cli::try_parse_from(["mahfel-api", "api-docs", "export"]).expect("command parses");
        assert_eq!(
            cli.command,
            Some(Command::ApiDocs {
                command: ApiDocsCommand::Export {
                    directory: PathBuf::from("specs"),
                },
            })
        );
    }

    #[test]
    fn accepts_an_api_docs_export_directory() {
        let cli = Cli::try_parse_from(["mahfel-api", "api-docs", "export", "contracts"])
            .expect("command parses");
        assert_eq!(
            cli.command,
            Some(Command::ApiDocs {
                command: ApiDocsCommand::Export {
                    directory: PathBuf::from("contracts"),
                },
            })
        );
    }

    #[test]
    fn rejects_unknown_api_docs_commands() {
        assert!(Cli::try_parse_from(["mahfel-api", "api-docs", "serve"]).is_err());
    }
}
