use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE: &str = ".manifest";

const SOURCE_FILES: &[&str] = &[
    "app_event_bindings.rs",
    "src/api/pack/import/mod.rs",
    "src/bin/export_app_events.rs",
    "src/error.rs",
    "src/event/mod.rs",
    "src/install/model.rs",
    "src/state/instance_types.rs",
    "src/state/process.rs",
];

pub fn tracked_inputs(manifest_dir: &Path) -> Vec<PathBuf> {
    let mut inputs = SOURCE_FILES
        .iter()
        .map(|path| manifest_dir.join(path))
        .collect::<Vec<_>>();
    inputs.push(manifest_dir.join("Cargo.toml"));
    inputs.push(manifest_dir.join("../..").join("Cargo.toml"));
    inputs
}

pub fn source_fingerprint(manifest_dir: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();

    for relative_path in SOURCE_FILES {
        let path = manifest_dir.join(relative_path);
        hash_input(&mut hasher, relative_path, &read(&path)?);
    }

    hash_matching_lines(
        &mut hasher,
        &manifest_dir.join("Cargo.toml"),
        &["ts-rs =", "export-ts =", "export-app-events"],
    )?;
    hash_matching_lines(
        &mut hasher,
        &manifest_dir.join("../..").join("Cargo.toml"),
        &["ts-rs ="],
    )?;

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn clear_generated_types(output: &Path) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(output).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().is_some_and(|extension| extension == "ts")
            || path.file_name().is_some_and(|name| name == MANIFEST_FILE)
        {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

pub fn write_manifest(
    manifest_dir: &Path,
    output: &Path,
) -> Result<(), String> {
    let source = source_fingerprint(manifest_dir)?;
    let files = generated_file_hashes(output)?;
    let mut manifest = format!("source {source}\n");

    for (name, hash) in files {
        manifest.push_str(&format!("{hash}  {name}\n"));
    }

    fs::write(output.join(MANIFEST_FILE), manifest)
        .map_err(|error| error.to_string())
}

pub fn validate_manifest(
    manifest_dir: &Path,
    output: &Path,
) -> Result<(), String> {
    let manifest_path = output.join(MANIFEST_FILE);
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|_| format!("{} is missing", manifest_path.display()))?;
    let mut lines = manifest.lines();
    let expected_source = lines
        .next()
        .and_then(|line| line.strip_prefix("source "))
        .ok_or_else(|| format!("{} is invalid", manifest_path.display()))?;
    let actual_source = source_fingerprint(manifest_dir)?;

    if actual_source != expected_source {
        return Err("the Rust event contract changed".to_string());
    }

    let mut expected_files = BTreeMap::new();
    for line in lines {
        let (hash, name) = line
            .split_once("  ")
            .ok_or_else(|| format!("{} is invalid", manifest_path.display()))?;
        expected_files.insert(name.to_string(), hash.to_string());
    }

    let actual_files = generated_file_hashes(output)?;
    if actual_files == expected_files {
        return Ok(());
    }

    let names = expected_files
        .keys()
        .chain(actual_files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let changed = names
        .into_iter()
        .filter(|name| expected_files.get(name) != actual_files.get(name))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("generated bindings changed: {changed}"))
}

fn generated_file_hashes(
    output: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(output).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if !path.extension().is_some_and(|extension| extension == "ts") {
            continue;
        }

        let name = path.file_name().and_then(|name| name.to_str()).ok_or_else(
            || format!("Invalid generated binding path: {}", path.display()),
        )?;
        files.insert(name.to_string(), sha256(&read(&path)?));
    }
    Ok(files)
}

fn hash_matching_lines(
    hasher: &mut Sha256,
    path: &Path,
    patterns: &[&str],
) -> Result<(), String> {
    let content =
        fs::read_to_string(path).map_err(|error| error.to_string())?;
    let selected = content
        .lines()
        .filter(|line| patterns.iter().any(|pattern| line.contains(pattern)))
        .collect::<Vec<_>>()
        .join("\n");
    hash_input(hasher, &path.display().to_string(), selected.as_bytes());
    Ok(())
}

fn hash_input(hasher: &mut Sha256, name: &str, content: &[u8]) {
    hasher.update(name.as_bytes());
    hasher.update([0]);
    hasher.update(content);
    hasher.update([0]);
}

fn sha256(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))
}
