use anyhow::Result;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Serve,
    Migrate,
    /// Apply development seeds. An explicit file or directory overrides the
    /// embedded defaults.
    Seed {
        path: Option<PathBuf>,
    },
}

pub(crate) fn command() -> Result<Command> {
    let mut arguments = std::env::args_os().skip(1);
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

fn reject_extra(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    name: &str,
) -> Result<()> {
    if arguments.next().is_some() {
        anyhow::bail!("unexpected extra arguments for `{name}`");
    }
    Ok(())
}
