use anyhow::Result;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Serve,
    Migrate,
    OpenApiExport {
        path: PathBuf,
    },
    /// Apply development seeds. An explicit file or directory overrides the
    /// embedded defaults.
    Seed {
        path: Option<PathBuf>,
    },
}

pub(crate) fn command() -> Result<Command> {
    parse(std::env::args_os().skip(1))
}

fn parse(mut arguments: impl Iterator<Item = std::ffi::OsString>) -> Result<Command> {
    let Some(command) = arguments.next() else {
        return Ok(Command::Serve);
    };
    match command.as_os_str() {
        value if value == OsStr::new("serve") => {
            reject_extra(&mut arguments, "serve")?;
            Ok(Command::Serve)
        }
        value if value == OsStr::new("migrate") => {
            reject_extra(&mut arguments, "migrate")?;
            Ok(Command::Migrate)
        }
        value if value == OsStr::new("openapi") => parse_openapi(&mut arguments),
        value if value == OsStr::new("seed") => {
            let path = arguments
                .next()
                .filter(|arg| !arg.is_empty())
                .map(PathBuf::from);
            reject_extra(&mut arguments, "seed")?;
            Ok(Command::Seed { path })
        }
        value => anyhow::bail!("unknown command: {}", value.to_string_lossy()),
    }
}

fn parse_openapi(arguments: &mut impl Iterator<Item = std::ffi::OsString>) -> Result<Command> {
    let Some(action) = arguments.next() else {
        anyhow::bail!("missing command after `openapi`; expected `export`");
    };
    if action != OsStr::new("export") {
        anyhow::bail!(
            "unknown openapi command: {}; expected `export`",
            action.to_string_lossy()
        );
    }
    let path = arguments
        .next()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("openapi/mahfel-api.json"));
    reject_extra(arguments, "openapi export")?;
    Ok(Command::OpenApiExport { path })
}

fn reject_extra(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    name: &str,
) -> Result<()> {
    if arguments.next().is_some() {
        anyhow::bail!("unexpected extra arguments for `{name}`");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Command, parse};
    use std::{ffi::OsString, path::PathBuf};

    fn parse_values(values: &[&str]) -> anyhow::Result<Command> {
        parse(values.iter().map(OsString::from))
    }

    #[test]
    fn defaults_openapi_export_to_the_repository_artifact() {
        assert_eq!(
            parse_values(&["openapi", "export"]).expect("command parses"),
            Command::OpenApiExport {
                path: PathBuf::from("openapi/mahfel-api.json"),
            }
        );
    }

    #[test]
    fn accepts_an_openapi_export_path() {
        assert_eq!(
            parse_values(&["openapi", "export", "contract.json"]).expect("command parses"),
            Command::OpenApiExport {
                path: PathBuf::from("contract.json"),
            }
        );
    }

    #[test]
    fn rejects_unknown_openapi_commands() {
        assert!(parse_values(&["openapi", "serve"]).is_err());
    }
}
