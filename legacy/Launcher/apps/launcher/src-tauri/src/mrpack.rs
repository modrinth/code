//! Modrinth `.mrpack` import / export (partial parity).
//!
//! Import: read `modrinth.index.json`, download listed files, extract
//! `overrides/` + `client-overrides/` into a new instance game dir.
//! Export: zip mods/resourcepacks/shaderpacks/datapacks/config as overrides
//! plus a minimal index (files without CDN mirrors stay override-only).
//!
//! Honest gap: no full Theseus content DB / hash cache; export does not upload
//! to Modrinth CDN — local-only pack sharing.

use crate::config;
use crate::instances::{self, DiskInstance};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const USER_AGENT: &str = concat!(
    "owyx/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/ebluffy/Owyx; mrpack)"
);
const MAX_MRPACK_BYTES: u64 = 512 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackFormat {
    game: String,
    #[allow(dead_code)]
    format_version: i32,
    #[allow(dead_code)]
    version_id: String,
    name: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    files: Vec<PackFile>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackFile {
    path: String,
    #[serde(default)]
    hashes: HashMap<String, String>,
    #[serde(default)]
    downloads: Vec<String>,
    #[serde(default)]
    file_size: u32,
    #[serde(default)]
    env: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPackFormat {
    game: &'static str,
    format_version: i32,
    version_id: String,
    name: String,
    summary: Option<String>,
    files: Vec<serde_json::Value>,
    dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackImportResult {
    pub instance: DiskInstance,
    pub name: String,
    pub files_downloaded: u32,
    pub overrides_extracted: u32,
    pub summary: Option<String>,
    /// True when some index files lacked downloads and were skipped.
    pub partial: bool,
    pub message: String,
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

fn assert_https_download_url(raw: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(raw).map_err(|e| format!("Bad URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Only https downloads allowed in mrpack".into());
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("Download URL must not include credentials".into());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "Missing host".to_string())?
        .to_ascii_lowercase();
    // Modrinth CDN + mirrors commonly used in packs.
    let allowed = host == "cdn.modrinth.com"
        || host.ends_with(".modrinth.com")
        || host == "github.com"
        || host == "objects.githubusercontent.com"
        || host == "raw.githubusercontent.com"
        || host.ends_with(".github.io");
    if !allowed {
        return Err(format!("Host not allowed for mrpack download: {host}"));
    }
    Ok(())
}

fn safe_rel(raw: &str) -> Result<PathBuf, String> {
    let normalized = raw.replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') || normalized.contains('\0') {
        return Err(format!("Unsafe pack path: {raw}"));
    }
    let path = Path::new(&normalized);
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(format!("Unsafe pack path: {raw}")),
        }
    }
    if out.as_os_str().is_empty() {
        return Err(format!("Empty pack path: {raw}"));
    }
    Ok(out)
}

fn hex_sha512(bytes: &[u8]) -> String {
    Sha512::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn verify_hashes(bytes: &[u8], hashes: &HashMap<String, String>) -> Result<(), String> {
    if let Some(expected) = hashes.get("sha512") {
        let actual = hex_sha512(bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!("SHA-512 mismatch: expected {expected}, got {actual}"));
        }
        return Ok(());
    }
    if let Some(expected) = hashes.get("sha1") {
        let actual = crate::install::hex_sha1(bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!("SHA-1 mismatch: expected {expected}, got {actual}"));
        }
        return Ok(());
    }
    Err("mrpack file missing sha512/sha1 hash — refusing to write unverified content".into())
}

fn deps_to_loader(deps: &HashMap<String, String>) -> Result<(String, String, String), String> {
    let mc = deps
        .get("minecraft")
        .cloned()
        .ok_or_else(|| "mrpack missing minecraft dependency".to_string())?;
    if let Some(v) = deps.get("fabric-loader") {
        return Ok(("fabric".into(), mc, v.clone()));
    }
    if let Some(v) = deps.get("quilt-loader") {
        return Ok(("quilt".into(), mc, v.clone()));
    }
    if let Some(v) = deps.get("neoforge") {
        return Ok(("neoforge".into(), mc, v.clone()));
    }
    if let Some(v) = deps.get("forge") {
        return Ok(("forge".into(), mc, v.clone()));
    }
    Ok(("vanilla".into(), mc, String::new()))
}

fn client_side_ok(file: &PackFile) -> bool {
    let Some(env) = &file.env else {
        return true;
    };
    match env.get("client").map(|s| s.as_str()) {
        Some("unsupported") => false,
        _ => true,
    }
}

async fn download_capped(url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    assert_https_download_url(url)?;
    let http = http_client()?;
    let res = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download HTTP: {e}"))?;
    assert_https_download_url(res.url().as_str())?;
    if let Some(len) = res.content_length() {
        if len > max_bytes {
            return Err(format!("File too large ({len} bytes)"));
        }
    }
    let mut out = Vec::new();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream: {e}"))?;
        if out.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err("File too large".into());
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

fn read_index_from_zip(bytes: &[u8]) -> Result<(PackFormat, ZipArchive<std::io::Cursor<&[u8]>>), String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor).map_err(|e| format!("Invalid mrpack zip: {e}"))?;
    let mut index_raw = String::new();
    {
        let mut entry = zip
            .by_name("modrinth.index.json")
            .map_err(|_| "No modrinth.index.json in mrpack".to_string())?;
        entry
            .read_to_string(&mut index_raw)
            .map_err(|e| format!("Read index: {e}"))?;
    }
    let pack: PackFormat =
        serde_json::from_str(&index_raw).map_err(|e| format!("Invalid modrinth.index.json: {e}"))?;
    if pack.game != "minecraft" {
        return Err("Pack does not support Minecraft".into());
    }
    Ok((pack, zip))
}

fn extract_overrides(
    zip: &mut ZipArchive<std::io::Cursor<&[u8]>>,
    game_dir: &Path,
) -> Result<u32, String> {
    let mut count = 0u32;
    let names: Vec<String> = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();
    for name in names {
        let prefix = if name.starts_with("overrides/") {
            "overrides/"
        } else if name.starts_with("client-overrides/") {
            "client-overrides/"
        } else {
            continue;
        };
        if name.ends_with('/') {
            continue;
        }
        let rel = name.trim_start_matches(prefix);
        if rel.is_empty() {
            continue;
        }
        let dest_rel = safe_rel(rel)?;
        let dest = game_dir.join(&dest_rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create override dir: {e}"))?;
        }
        let mut entry = zip
            .by_name(&name)
            .map_err(|e| format!("Open override {name}: {e}"))?;
        let mut out = File::create(&dest).map_err(|e| format!("Write override: {e}"))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("Extract override: {e}"))?;
        count += 1;
    }
    Ok(count)
}

/// Import a local `.mrpack` into a new instance (create + overlay files).
pub async fn import_mrpack_file(path: String) -> Result<MrpackImportResult, String> {
    let path = PathBuf::from(path.trim());
    if !path.is_file() {
        return Err("mrpack file not found".into());
    }
    let meta = fs::metadata(&path).map_err(|e| format!("Stat mrpack: {e}"))?;
    if meta.len() > MAX_MRPACK_BYTES {
        return Err("mrpack file too large".into());
    }
    let bytes = fs::read(&path).map_err(|e| format!("Read mrpack: {e}"))?;
    install_mrpack_bytes(&bytes, None).await
}

/// Download a Modrinth modpack version (primary `.mrpack`) and import it.
pub async fn import_mrpack_from_modrinth(
    project_id: String,
    version_id: Option<String>,
) -> Result<MrpackImportResult, String> {
    let version = if let Some(vid) = version_id.filter(|s| !s.trim().is_empty()) {
        crate::modrinth::fetch_version_public(&vid).await?
    } else {
        let versions =
            crate::modrinth::list_project_versions(project_id.clone(), None, None).await?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| "No modpack versions found".to_string())?
    };
    let file = version
        .files
        .iter()
        .find(|f| f.primary || f.filename.ends_with(".mrpack"))
        .cloned()
        .or_else(|| version.files.first().cloned())
        .ok_or_else(|| "Modpack version has no files".to_string())?;
    if !file.filename.ends_with(".mrpack") && !file.filename.ends_with(".zip") {
        return Err(format!(
            "Expected .mrpack file, got {}",
            file.filename
        ));
    }
    crate::modrinth::assert_download_url_public(&file.url)?;
    let bytes = crate::modrinth::download_bytes_public(&file.url, MAX_MRPACK_BYTES).await?;
    install_mrpack_bytes(&bytes, Some(version.name.as_str())).await
}

async fn install_mrpack_bytes(
    bytes: &[u8],
    name_override: Option<&str>,
) -> Result<MrpackImportResult, String> {
    let (pack, mut zip) = read_index_from_zip(bytes)?;
    let (loader, mc, loader_version) = deps_to_loader(&pack.dependencies)?;
    let name = name_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(pack.name.as_str());
    let instance = instances::prepare_instance(name, &loader, &mc, &loader_version, None)?;
    let instance_id = instance.id.clone();
    let result = install_mrpack_into_prepared(&pack, &mut zip, &instance).await;
    match result {
        Ok(ok) => Ok(ok),
        Err(err) => {
            let _ = instances::delete_instance(&instance_id);
            Err(err)
        }
    }
}

async fn install_mrpack_into_prepared(
    pack: &PackFormat,
    zip: &mut ZipArchive<std::io::Cursor<&[u8]>>,
    instance: &DiskInstance,
) -> Result<MrpackImportResult, String> {
    let game_dir = PathBuf::from(&instance.path).join("game");
    fs::create_dir_all(&game_dir).map_err(|e| format!("Create game dir: {e}"))?;

    let mut downloaded = 0u32;
    let mut skipped = 0u32;
    for file in &pack.files {
        if !client_side_ok(file) {
            continue;
        }
        let rel = safe_rel(&file.path)?;
        let dest = game_dir.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create pack dir: {e}"))?;
        }
        let Some(url) = file.downloads.iter().find(|u| assert_https_download_url(u).is_ok()) else {
            skipped += 1;
            continue;
        };
        let max = if file.file_size > 0 {
            (file.file_size as u64)
                .saturating_mul(2)
                .max(1024)
                .min(MAX_FILE_BYTES)
        } else {
            MAX_FILE_BYTES
        };
        let file_bytes = download_capped(url, max).await?;
        verify_hashes(&file_bytes, &file.hashes)?;
        config::atomic_write_path(&dest, &file_bytes)?;
        downloaded += 1;
    }

    let overrides = extract_overrides(zip, &game_dir)?;
    let partial = skipped > 0;
    let message = if partial {
        format!(
            "Imported {} with {downloaded} files and {overrides} overrides ({skipped} index files skipped — no allowed download URL)",
            pack.name
        )
    } else {
        format!(
            "Imported {} with {downloaded} files and {overrides} overrides",
            pack.name
        )
    };

    Ok(MrpackImportResult {
        instance: instance.clone(),
        name: pack.name.clone(),
        files_downloaded: downloaded,
        overrides_extracted: overrides,
        summary: pack.summary.clone(),
        partial,
        message,
    })
}

/// Export instance content folders into a local `.mrpack` (override-heavy, partial).
pub fn export_mrpack(instance_id: String, dest_path: String) -> Result<String, String> {
    instances::validate_instance_id(&instance_id)?;
    let list = instances::list_instances()?;
    let inst = list
        .into_iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    let game_dir = PathBuf::from(&inst.path).join("game");
    if !game_dir.is_dir() {
        return Err("Instance game directory missing".into());
    }
    let dest = PathBuf::from(dest_path.trim());
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create export dir: {e}"))?;
    }

    let mut dependencies = HashMap::new();
    dependencies.insert("minecraft".into(), inst.minecraft.clone());
    match inst.loader.as_str() {
        "fabric" => {
            dependencies.insert("fabric-loader".into(), inst.loader_version.clone());
        }
        "quilt" => {
            dependencies.insert("quilt-loader".into(), inst.loader_version.clone());
        }
        "forge" => {
            dependencies.insert("forge".into(), inst.loader_version.clone());
        }
        "neoforge" => {
            dependencies.insert("neoforge".into(), inst.loader_version.clone());
        }
        _ => {}
    }

    let prefixes = ["mods", "resourcepacks", "shaderpacks", "datapacks", "config"];
    let mut file = File::create(&dest).map_err(|e| format!("Create mrpack: {e}"))?;
    let mut zip = ZipWriter::new(&mut file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut exported_files = 0u32;

    for prefix in prefixes {
        let dir = game_dir.join(prefix);
        if !dir.is_dir() {
            continue;
        }
        let mut stack = vec![dir.clone()];
        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current).map_err(|e| format!("Read dir: {e}"))? {
                let entry = entry.map_err(|e| format!("Read entry: {e}"))?;
                let ft = entry.file_type().map_err(|e| format!("file type: {e}"))?;
                if ft.is_dir() {
                    stack.push(entry.path());
                    continue;
                }
                if !ft.is_file() {
                    continue;
                }
                let path = entry.path();
                let rel = path
                    .strip_prefix(&game_dir)
                    .map_err(|_| "Path escaped game dir".to_string())?;
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                if rel_str.contains("..") {
                    continue;
                }
                let zip_path = format!("overrides/{rel_str}");
                zip.start_file(&zip_path, options)
                    .map_err(|e| format!("Zip start: {e}"))?;
                let mut bytes = Vec::new();
                File::open(&path)
                    .and_then(|mut f| f.read_to_end(&mut bytes))
                    .map_err(|e| format!("Read {}: {e}", path.display()))?;
                zip.write_all(&bytes).map_err(|e| format!("Zip write: {e}"))?;
                exported_files += 1;
            }
        }
    }

    let index = ExportPackFormat {
        game: "minecraft",
        format_version: 1,
        version_id: format!("owyx-export-{}", inst.id),
        name: inst.name.clone(),
        summary: Some(
            "Exported from Owyx Launcher. Files are in overrides/ (no CDN mirrors).".into(),
        ),
        files: vec![],
        dependencies,
    };
    let index_json =
        serde_json::to_vec_pretty(&index).map_err(|e| format!("Serialize index: {e}"))?;
    zip.start_file("modrinth.index.json", options)
        .map_err(|e| format!("Zip index: {e}"))?;
    zip.write_all(&index_json)
        .map_err(|e| format!("Write index: {e}"))?;
    zip.finish().map_err(|e| format!("Finish zip: {e}"))?;

    Ok(format!(
        "Exported {} override files to {}",
        exported_files,
        dest.display()
    ))
}

/// Install mrpack bytes into an existing instance (catalog overlay path).
pub async fn install_mrpack_bytes_into_instance(
    instance_id: &str,
    bytes: &[u8],
) -> Result<(), String> {
    instances::validate_instance_id(instance_id)?;
    let (pack, mut zip) = read_index_from_zip(bytes)?;
    let game_dir = config::ensure_layout()?
        .join("instances")
        .join(instance_id)
        .join("game");
    fs::create_dir_all(&game_dir).map_err(|e| format!("Create game dir: {e}"))?;
    for file in &pack.files {
        if !client_side_ok(file) {
            continue;
        }
        let rel = safe_rel(&file.path)?;
        let dest = game_dir.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create pack dir: {e}"))?;
        }
        let Some(url) = file.downloads.iter().find(|u| assert_https_download_url(u).is_ok()) else {
            continue;
        };
        let file_bytes = download_capped(url, MAX_FILE_BYTES).await?;
        verify_hashes(&file_bytes, &file.hashes)?;
        config::atomic_write_path(&dest, &file_bytes)?;
    }
    let _ = extract_overrides(&mut zip, &game_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deps_fabric() {
        let mut d = HashMap::new();
        d.insert("minecraft".into(), "1.21.1".into());
        d.insert("fabric-loader".into(), "0.16.0".into());
        let (l, mc, v) = deps_to_loader(&d).unwrap();
        assert_eq!(l, "fabric");
        assert_eq!(mc, "1.21.1");
        assert_eq!(v, "0.16.0");
    }

    #[test]
    fn safe_rel_blocks_traversal() {
        assert!(safe_rel("../x").is_err());
        assert!(safe_rel("mods/foo.jar").is_ok());
    }

    #[test]
    fn verify_hashes_requires_sha() {
        let mut empty = HashMap::new();
        assert!(verify_hashes(b"abc", &empty).is_err());
        empty.insert("sha512".into(), "deadbeef".into());
        assert!(verify_hashes(b"abc", &empty).is_err());
        let good = hex_sha512(b"abc");
        empty.insert("sha512".into(), good);
        assert!(verify_hashes(b"abc", &empty).is_ok());
    }
}
