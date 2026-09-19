use anyhow::Result;
use std::ffi::OsStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Serve,
    Migrate,
}

pub(crate) fn command() -> Result<Command> {
    let mut arguments = std::env::args_os().skip(1);
    let command = match arguments.next().as_deref() {
        None => Command::Serve,
        Some(value) if value == OsStr::new("serve") => Command::Serve,
        Some(value) if value == OsStr::new("migrate") => Command::Migrate,
        Some(value) => anyhow::bail!("unknown command: {}", value.to_string_lossy()),
    };
    if arguments.next().is_some() {
        anyhow::bail!("expected at most one command: serve or migrate");
    }
    Ok(command)
}
