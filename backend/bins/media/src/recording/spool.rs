use std::path::PathBuf;

pub(super) fn paths(root: PathBuf, extension: Option<&str>) -> std::io::Result<Vec<PathBuf>> {
    let mut pending = vec![root];
    let mut result = Vec::new();
    while let Some(path) = pending.pop() {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                pending.push(entry.path());
            } else if extension.is_none_or(|extension| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == extension)
            }) {
                result.push(entry.path());
            }
        }
    }
    result.sort();
    Ok(result)
}
pub(super) async fn directory_size(root: PathBuf) -> anyhow::Result<u64> {
    tokio::task::spawn_blocking(move || {
        paths(root, None)?
            .into_iter()
            .try_fold(0_u64, |size, path| {
                Ok::<_, std::io::Error>(size.saturating_add(std::fs::metadata(path)?.len()))
            })
    })
    .await?
    .map_err(Into::into)
}
pub(super) async fn has_journals(root: PathBuf) -> anyhow::Result<bool> {
    Ok(
        !tokio::task::spawn_blocking(move || paths(root, Some("receipt")))
            .await??
            .is_empty(),
    )
}

pub(super) async fn remove_unjournaled(root: PathBuf) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        for path in paths(root, None)? {
            if path
                .extension()
                .is_some_and(|value| value == "webm" || value == "mkv")
                && !path.with_extension("receipt").exists()
            {
                match std::fs::remove_file(path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
        Ok::<_, std::io::Error>(())
    })
    .await??;
    Ok(())
}
