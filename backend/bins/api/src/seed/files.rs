use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/seeds.rs"));
}

pub(super) struct SeedSource {
    pub name: String,
    pub content: String,
    pub base_dir: Option<PathBuf>,
}

#[cfg(test)]
impl From<(String, String)> for SeedSource {
    fn from((name, content): (String, String)) -> Self {
        Self {
            name,
            content,
            base_dir: None,
        }
    }
}

pub(super) fn embedded_seeds() -> Vec<SeedSource> {
    embedded::SEEDS
        .iter()
        .map(|(name, content)| SeedSource {
            name: (*name).to_owned(),
            content: (*content).to_owned(),
            base_dir: None,
        })
        .collect()
}

pub(super) fn read_path(path: &Path) -> Result<Vec<SeedSource>> {
    if path.is_dir() {
        let files = seed_files(path)?;
        if files.is_empty() {
            anyhow::bail!("no .toml seeds found in {}", path.display());
        }
        files.iter().map(|file| read_file(file)).collect()
    } else if path.is_file() {
        if path.extension().is_none_or(|extension| extension != "toml") {
            anyhow::bail!("seed file must have a .toml extension: {}", path.display());
        }
        Ok(vec![read_file(path)?])
    } else {
        anyhow::bail!("seed path does not exist: {}", path.display())
    }
}

fn seed_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(directory)
        .with_context(|| format!("read seed directory {}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.retain(|path| {
        path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "toml")
    });
    files.sort();
    Ok(files)
}

fn read_file(path: &Path) -> Result<SeedSource> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("seed file name is not valid UTF-8")?
        .to_owned();
    let content =
        fs::read_to_string(path).with_context(|| format!("read seed {}", path.display()))?;
    Ok(SeedSource {
        name,
        content,
        base_dir: Some(
            path.canonicalize()?
                .parent()
                .context("seed has no parent")?
                .to_owned(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::read_path;
    use crate::seed::fixture::parse_seeds;

    #[test]
    fn explicit_paths_select_only_toml_and_validate_all_files() {
        let directory = std::env::temp_dir().join(format!("mahfel-seed-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).expect("create test seed directory");
        let valid = directory.join("0001_principal.toml");
        fs::write(
            &valid,
            "kind = 'development_principal'\nid = '00000000-0000-0000-0000-000000000001'\nusername = 'dev'\nemail = 'dev@mahfel.local'\ndisplay_name = 'Dev'\n",
        )
        .expect("write valid seed");
        let sql = directory.join("0002_legacy.sql");
        fs::write(&sql, "DELETE FROM mahfel.users;").expect("write non-TOML file");

        let sources = read_path(&directory).expect("read TOML seeds");
        assert_eq!(sources.len(), 1);
        assert!(parse_seeds(sources).is_ok());
        assert!(read_path(&sql).is_err());

        let invalid = directory.join("0003_invalid.toml");
        fs::write(&invalid, "kind = 'unsupported'").expect("write invalid seed");
        assert!(parse_seeds(read_path(&directory).expect("read all TOML seeds")).is_err());
        fs::remove_dir_all(directory).expect("remove test seed directory");
    }
}
