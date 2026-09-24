use std::{env, error::Error, fs, path::PathBuf};

/// Embed every `backend/seeds/*.toml` file at build time, sorted by file name.
///
/// Build-time embedding keeps the runtime image free of seed files while still
/// discovering seeds by directory, so adding a new fixture needs no code change.
fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let seeds_dir = manifest_dir.join("../../seeds");
    println!("cargo:rerun-if-changed={}", seeds_dir.display());

    let mut seeds: Vec<PathBuf> = fs::read_dir(&seeds_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    seeds.retain(|path| {
        path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "toml")
    });
    if seeds.is_empty() {
        return Err("no TOML seed files found".into());
    }
    seeds.sort();

    let mut generated = String::from("pub(crate) const SEEDS: &[(&str, &str)] = &[\n");
    for path in &seeds {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("seed file name is not valid UTF-8")?;
        let absolute = path.to_str().ok_or("seed path is not valid UTF-8")?;
        generated.push_str(&format!("    ({name:?}, include_str!({absolute:?})),\n"));
    }
    generated.push_str("];\n");

    let out = PathBuf::from(env::var("OUT_DIR")?).join("seeds.rs");
    fs::write(out, generated)?;
    Ok(())
}
