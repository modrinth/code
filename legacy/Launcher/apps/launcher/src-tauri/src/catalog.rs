//! Site catalog: list servers/packs, download http_zip / manifest overlay,
//! write servers.dat for Play. Players never see SFTP credentials.

use crate::config;
use crate::install;
use crate::instances;
use crate::owyx;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

const USER_AGENT: &str = concat!("Owyx/", env!("CARGO_PKG_VERSION"));
const MAX_PACK_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PACK_FILES: usize = 50_000;
const MAX_REDIRECTS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPack {
    pub id: String,
    pub name: String,
    pub minecraft: String,
    pub loader: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub source_type: String,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub manifest_url: Option<String>,
    #[serde(default)]
    pub download_available: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogServer {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub pack_id: Option<String>,
    #[serde(default)]
    pub minecraft: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub requires_account: bool,
    #[serde(default)]
    pub pack: Option<CatalogPack>,
}

fn default_port() -> u16 {
    25565
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogLoadResult {
    pub servers: Vec<CatalogServer>,
    pub packs: Vec<CatalogPack>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogInstallResult {
    pub instance_id: String,
    pub ready: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMeta {
    pub server_id: String,
    pub pack_id: Option<String>,
    pub address: String,
    pub port: u16,
    #[serde(default)]
    pub pack_sha256: Option<String>,
}

pub(crate) fn http_client(api_base: &str) -> Result<reqwest::Client, String> {
    let api_base = api_base.to_string();
    let redirect_policy = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            return attempt.error("Too many redirects");
        }
        let Some(initial) = attempt.previous().first() else {
            return attempt.error("Redirect has no source URL");
        };
        if redirect_allowed(initial, attempt.url(), &api_base) {
            attempt.follow()
        } else {
            attempt.error("Unsafe download redirect")
        }
    });
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            if let Some(key) = owyx::resolve_client_key() {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(owyx::CLIENT_KEY_HEADER.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&key),
                ) {
                    headers.insert(name, val);
                }
            }
            headers
        })
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .redirect(redirect_policy)
        .build()
        .map_err(|e| format!("network: {e}"))
}

fn is_loopback_or_private(host: &str) -> bool {
    if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]" {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v) => v.is_loopback() || v.is_private() || v.is_link_local(),
            IpAddr::V6(v) => v.is_loopback() || v.is_unique_local(),
        };
    }
    false
}

fn is_owyx_host(host: &str) -> bool {
    host == "owyx.site"
        || host == "api.owyx.site"
        || host == "mc.owyx.site"
        || host.ends_with(".owyx.site")
}

fn redirect_allowed(initial: &reqwest::Url, next: &reqwest::Url, api_base: &str) -> bool {
    if next.username() != "" || next.password().is_some() {
        return false;
    }
    if !matches!(next.scheme(), "http" | "https") {
        return false;
    }
    // Never let a trusted HTTPS URL silently downgrade to clear-text HTTP.
    if initial.scheme() == "https" && next.scheme() != "https" {
        return false;
    }
    let Some(initial_host) = initial.host_str().map(str::to_ascii_lowercase) else {
        return false;
    };
    let Some(next_host) = next.host_str().map(str::to_ascii_lowercase) else {
        return false;
    };
    let api_host = reqwest::Url::parse(api_base)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase));
    let initial_is_private =
        is_loopback_or_private(&initial_host) || api_host.as_deref() == Some(initial_host.as_str());
    let next_is_private = is_loopback_or_private(&next_host);

    // A public warehouse must not redirect into localhost, LAN, or link-local
    // services. Direct private/API URLs remain available for local development.
    !next_is_private || initial_is_private || api_host.as_deref() == Some(next_host.as_str())
}

/// Players: HTTPS on the public internet; HTTP only for loopback/private + API host.
/// No credentials in the URL. Decision recorded in PLAN.md.
pub fn assert_pack_download_url(raw: &str, api_base: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(raw).map_err(|e| format!("Bad pack URL: {e}"))?;
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("Download URL must not include credentials".into());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "Missing host".to_string())?
        .to_ascii_lowercase();
    let api_host = reqwest::Url::parse(api_base)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_ascii_lowercase()));
    let same_api = api_host.as_deref() == Some(host.as_str());
    match parsed.scheme() {
        "https" => {
            if is_loopback_or_private(&host) || is_owyx_host(&host) || same_api {
                return Ok(());
            }
            // Public warehouse HTTPS is allowed (mini-PC behind FRPC / object storage).
            Ok(())
        }
        "http" => {
            if is_loopback_or_private(&host) || same_api || is_owyx_host(&host) {
                return Ok(());
            }
            Err(format!(
                "HTTP pack downloads only from local/private or API host, not {host}"
            ))
        }
        other => Err(format!("Unsupported pack URL scheme: {other}")),
    }
}

pub async fn list_catalog(base_url: Option<String>) -> Result<CatalogLoadResult, String> {
    let base = owyx::resolve_base(base_url.as_deref());
    let client = http_client(&base)?;
    let servers_res = client
        .get(format!("{base}/api/launcher/v1/servers"))
        .send()
        .await
        .map_err(|_| "network".to_string())?;
    let packs_res = client
        .get(format!("{base}/api/launcher/v1/packs"))
        .send()
        .await
        .map_err(|_| "network".to_string())?;

    if !servers_res.status().is_success() {
        return Ok(CatalogLoadResult {
            servers: Vec::new(),
            packs: Vec::new(),
            error: Some("server".into()),
        });
    }
    let s_body: serde_json::Value = servers_res.json().await.map_err(|_| "server".to_string())?;
    let p_body: serde_json::Value = if packs_res.status().is_success() {
        packs_res.json().await.unwrap_or(serde_json::json!({}))
    } else {
        crate::log::warn(&format!(
            "Catalog packs request returned HTTP {}",
            packs_res.status()
        ));
        serde_json::json!({})
    };
    let servers: Vec<CatalogServer> = serde_json::from_value(
        s_body
            .get("servers")
            .cloned()
            .unwrap_or(serde_json::json!([])),
    )
    .unwrap_or_default();
    let packs: Vec<CatalogPack> = serde_json::from_value(
        p_body
            .get("packs")
            .cloned()
            .unwrap_or(serde_json::json!([])),
    )
    .unwrap_or_default();
    Ok(CatalogLoadResult {
        servers,
        packs,
        error: None,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn install_catalog_server(
    app: AppHandle,
    server_id: String,
    name: String,
    address: String,
    port: u16,
    pack_id: Option<String>,
    minecraft: String,
    loader: String,
    download_url: Option<String>,
    sha256: Option<String>,
    manifest_url: Option<String>,
    source_type: String,
) -> Result<CatalogInstallResult, String> {
    let loader_version = if loader.eq_ignore_ascii_case("vanilla") {
        String::new()
    } else {
        crate::versions::list_loader_versions(&loader, &minecraft)
            .await
            .ok()
            .and_then(|v| v.into_iter().next())
            .unwrap_or_default()
    };
    let instance_id = existing_or_prepare(&name, &loader, &minecraft, &loader_version, &server_id)?;
    write_catalog_meta(
        &instance_id,
        CatalogMeta {
            server_id: server_id.clone(),
            pack_id: pack_id.clone(),
            address: address.clone(),
            port,
            pack_sha256: sha256.clone(),
        },
    )?;

    emit(
        &app,
        &instance_id,
        "install",
        0,
        3,
        "Installing Minecraft…".into(),
    )?;

    install::install_instance(
        app.clone(),
        instance_id.clone(),
        name.clone(),
        loader.clone(),
        minecraft.clone(),
        loader_version,
    )
    .await?;

    emit(
        &app,
        &instance_id,
        "overlay",
        1,
        3,
        "Downloading pack…".into(),
    )?;

    let base = owyx::resolve_base(None);
    let overlay = download_overlay(
        &app,
        &instance_id,
        &base,
        &source_type,
        download_url.as_deref(),
        sha256.as_deref(),
        manifest_url.as_deref(),
    )
    .await;
    if let Err(err) = overlay {
        crate::log::error(&format!(
            "Catalog overlay failed for instance {instance_id}: {err}"
        ));
        return Err(match mark_error(&instance_id) {
            Ok(()) => err,
            Err(cleanup) => format!("{err}; failed to mark instance as broken: {cleanup}"),
        });
    }

    let game_dir = config::ensure_layout()?
        .join("instances")
        .join(&instance_id)
        .join("game");
    if let Err(err) = write_servers_dat(&game_dir, &name, &address, port) {
        return Err(match mark_error(&instance_id) {
            Ok(()) => err,
            Err(cleanup) => format!("{err}; failed to mark instance as broken: {cleanup}"),
        });
    }
    emit(&app, &instance_id, "done", 3, 3, "Pack ready".into())?;

    Ok(CatalogInstallResult {
        instance_id,
        ready: true,
        message: "ready".into(),
    })
}

fn existing_or_prepare(
    name: &str,
    loader: &str,
    minecraft: &str,
    loader_version: &str,
    server_id: &str,
) -> Result<String, String> {
    let preferred = format!("srv-{server_id}");
    if let Ok(root) = instance_root_if_exists(&preferred) {
        if root.join("instance.json").is_file() {
            return Ok(preferred);
        }
    }
    // Reuse any instance that already points at this server.
    if let Some(id) = find_instance_for_server(server_id)? {
        return Ok(id);
    }
    let disk = instances::prepare_instance(name, loader, minecraft, loader_version, None)?;
    // Rename folder to srv-{id} when possible so Play can find it next time.
    if disk.id != preferred {
        if let Ok(from) = instance_root_if_exists(&disk.id) {
            let to = config::ensure_layout()?.join("instances").join(&preferred);
            if !to.exists() {
                if fs::rename(&from, &to).is_ok() {
                    if let Ok(raw) = fs::read_to_string(to.join("instance.json")) {
                        if let Ok(mut rec) = serde_json::from_str::<serde_json::Value>(&raw) {
                            rec["id"] = serde_json::Value::String(preferred.clone());
                            let _ = fs::write(
                                to.join("instance.json"),
                                serde_json::to_string_pretty(&rec).unwrap_or(raw),
                            );
                        }
                    }
                    return Ok(preferred);
                }
            }
        }
        return Ok(disk.id);
    }
    Ok(disk.id)
}

fn instance_root_if_exists(id: &str) -> Result<PathBuf, String> {
    instances::validate_instance_id(id)?;
    Ok(config::ensure_layout()?.join("instances").join(id))
}

fn find_instance_for_server(server_id: &str) -> Result<Option<String>, String> {
    let root = config::ensure_layout()?.join("instances");
    let Ok(entries) = fs::read_dir(&root) else {
        return Ok(None);
    };
    for entry in entries.flatten() {
        let meta = entry.path().join("catalog.json");
        if !meta.is_file() {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&meta) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<CatalogMeta>(&raw) else {
            continue;
        };
        if parsed.server_id == server_id {
            if let Some(name) = entry.file_name().to_str() {
                return Ok(Some(name.to_string()));
            }
        }
    }
    Ok(None)
}

fn write_catalog_meta(instance_id: &str, meta: CatalogMeta) -> Result<(), String> {
    let root = instance_root_if_exists(instance_id)?;
    fs::create_dir_all(&root).map_err(|e| format!("Create instance: {e}"))?;
    let raw = serde_json::to_string_pretty(&meta).map_err(|e| format!("Serialize catalog: {e}"))?;
    crate::config::atomic_write_path(&root.join("catalog.json"), raw.as_bytes())
}

pub fn read_catalog_meta(instance_id: &str) -> Option<CatalogMeta> {
    let root = instance_root_if_exists(instance_id).ok()?;
    let raw = fs::read_to_string(root.join("catalog.json")).ok()?;
    serde_json::from_str(&raw).ok()
}

fn mark_error(instance_id: &str) -> Result<(), String> {
    let root = instance_root_if_exists(instance_id)?;
    mark_error_at(&root)
}

fn mark_error_at(root: &Path) -> Result<(), String> {
    let meta_path = root.join("meta.json");
    if meta_path.exists() {
        fs::remove_file(&meta_path).map_err(|e| format!("Remove stale meta.json: {e}"))?;
    }
    let instance_path = root.join("instance.json");
    let raw = fs::read_to_string(&instance_path).map_err(|e| format!("Read instance.json: {e}"))?;
    let mut rec: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("Parse instance.json: {e}"))?;
    rec["status"] = serde_json::Value::String("error".into());
    let updated =
        serde_json::to_vec_pretty(&rec).map_err(|e| format!("Serialize instance.json: {e}"))?;
    config::atomic_write_path(&instance_path, &updated)
}

async fn download_overlay(
    app: &AppHandle,
    instance_id: &str,
    api_base: &str,
    source_type: &str,
    download_url: Option<&str>,
    sha256: Option<&str>,
    manifest_url: Option<&str>,
) -> Result<(), String> {
    let instance_dir = config::ensure_layout()?.join("instances").join(instance_id);
    let game_dir = instance_dir.join("game");
    fs::create_dir_all(&game_dir).map_err(|e| format!("Create game dir: {e}"))?;
    let staging_dir = instance_dir.join(".catalog-overlay");
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(|e| format!("Clean overlay staging: {e}"))?;
    }
    fs::create_dir_all(&staging_dir).map_err(|e| format!("Create overlay staging: {e}"))?;

    let result = async {
        if source_type == "sftp" {
            return Err("SFTP is admin-only. Publish an HTTP zip first.".into());
        }
        if source_type == "mrpack" {
            let url = download_url
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "mrpack download URL is missing".to_string())?;
            assert_pack_download_url(url, api_base)?;
            emit(
                app,
                instance_id,
                "download",
                1,
                3,
                format!("Downloading mrpack {url}"),
            )?;
            let bytes = download_capped(url, api_base).await?;
            if let Some(expected) = sha256.filter(|s| !s.is_empty()) {
                let actual = hex_sha256(&bytes);
                if !actual.eq_ignore_ascii_case(expected) {
                    return Err(format!(
                        "SHA256 mismatch: expected {expected}, got {actual}"
                    ));
                }
            }
            // Write bytes to a temp file path-free install via mrpack module.
            crate::mrpack::install_mrpack_bytes_into_instance(instance_id, &bytes).await?;
            return Ok(());
        }
        if source_type == "http_manifest" {
            let url = manifest_url
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "Manifest URL is missing".to_string())?;
            download_manifest_files(app, instance_id, api_base, url, &staging_dir).await?;
            return apply_staged_overlay(&staging_dir, &game_dir);
        }
        let Some(url) = download_url.filter(|s| !s.is_empty()) else {
            // No overlay is OK (vanilla + servers.dat only).
            return Ok(());
        };
        assert_pack_download_url(url, api_base)?;
        emit(
            app,
            instance_id,
            "download",
            1,
            3,
            format!("Downloading {url}"),
        )?;
        let bytes = download_capped(url, api_base).await?;
        if let Some(expected) = sha256.filter(|s| !s.is_empty()) {
            let actual = hex_sha256(&bytes);
            if !actual.eq_ignore_ascii_case(expected) {
                return Err(format!(
                    "SHA256 mismatch: expected {expected}, got {actual}"
                ));
            }
        }
        extract_zip_overlay(&bytes, &staging_dir)?;
        apply_staged_overlay(&staging_dir, &game_dir)
    };
    let outcome = result.await;
    let cleanup = fs::remove_dir_all(&staging_dir);
    match (outcome, cleanup) {
        (Err(err), _) => Err(err),
        (Ok(()), Err(err)) => Err(format!("Clean overlay staging: {err}")),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    path: Option<String>,
    url: Option<String>,
    sha256: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PackManifestBody {
    #[serde(default)]
    files: Vec<ManifestFile>,
}

async fn download_manifest_files(
    app: &AppHandle,
    instance_id: &str,
    api_base: &str,
    manifest_url: &str,
    game_dir: &Path,
) -> Result<(), String> {
    assert_pack_download_url(manifest_url, api_base)?;
    let client = http_client(api_base)?;
    let res = client
        .get(manifest_url)
        .send()
        .await
        .map_err(|e| format!("Manifest: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Manifest HTTP: {e}"))?;
    assert_pack_download_url(res.url().as_str(), api_base)?;
    let body: PackManifestBody = res
        .json()
        .await
        .map_err(|e| format!("Manifest JSON: {e}"))?;
    let total = body.files.len() as u32;
    for (i, file) in body.files.iter().enumerate() {
        let url = file
            .url
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("Manifest file {} has no URL", i + 1))?;
        let rel = file
            .path
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("Manifest file {} has no path", i + 1))?;
        emit(
            app,
            instance_id,
            "download",
            i as u32,
            total.max(1),
            format!("Downloading {rel}"),
        )?;
        let bytes = download_capped(url, api_base).await?;
        if let Some(expected_size) = file.size {
            if bytes.len() as u64 != expected_size {
                return Err(format!(
                    "Size mismatch for {rel}: expected {expected_size}, got {}",
                    bytes.len()
                ));
            }
        }
        if let Some(expected) = file.sha256.as_deref().filter(|s| !s.is_empty()) {
            let actual = hex_sha256(&bytes);
            if !actual.eq_ignore_ascii_case(expected) {
                return Err(format!("SHA256 mismatch for {rel}"));
            }
        }
        if url.ends_with(".zip") || rel.ends_with(".zip") {
            extract_zip_overlay(&bytes, game_dir)?;
        } else {
            let dest = game_dir.join(safe_rel(rel)?);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Create parent: {e}"))?;
            }
            crate::config::atomic_write_path(&dest, &bytes)?;
        }
    }
    Ok(())
}

async fn download_capped(url: &str, api_base: &str) -> Result<Vec<u8>, String> {
    use futures_util::StreamExt;
    assert_pack_download_url(url, api_base)?;
    let client = http_client(api_base)?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download HTTP: {e}"))?;
    assert_pack_download_url(res.url().as_str(), api_base)?;
    if let Some(len) = res.content_length() {
        if len > MAX_PACK_BYTES {
            return Err(format!("File too large ({len} bytes)"));
        }
    }
    let mut out = Vec::new();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream: {e}"))?;
        if out.len() as u64 + chunk.len() as u64 > MAX_PACK_BYTES {
            return Err(format!("File exceeded {MAX_PACK_BYTES} bytes"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

fn extract_zip_overlay(bytes: &[u8], game_dir: &Path) -> Result<(), String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Zip: {e}"))?;
    if archive.len() > MAX_PACK_FILES {
        return Err(format!("Zip has too many entries ({})", archive.len()));
    }
    let mut unpacked_bytes = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Zip entry: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(format!(
                "Zip symlink entries are not allowed: {}",
                entry.name()
            ));
        }
        unpacked_bytes = unpacked_bytes
            .checked_add(entry.size())
            .ok_or_else(|| "Zip unpacked size overflow".to_string())?;
        if unpacked_bytes > MAX_PACK_BYTES {
            return Err(format!(
                "Zip expands past the {MAX_PACK_BYTES} byte safety limit"
            ));
        }
        let name = entry.name().to_string();
        if is_meta_inf_path(&name) {
            continue;
        }
        let rel = safe_rel(&name)?;
        let dest = game_dir.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create overlay dir: {e}"))?;
        }
        let mut out = File::create(&dest).map_err(|e| format!("Write overlay: {e}"))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("Extract: {e}"))?;
    }
    Ok(())
}

fn is_meta_inf_path(raw: &str) -> bool {
    raw.replace('\\', "/")
        .split('/')
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case("META-INF"))
}

fn apply_staged_overlay(staging_dir: &Path, game_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(game_dir).map_err(|e| format!("Create game dir: {e}"))?;
    if fs::symlink_metadata(game_dir)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err("Game directory must not be a symlink".into());
    }

    let mut pending = vec![staging_dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| format!("Read overlay staging: {e}"))? {
            let entry = entry.map_err(|e| format!("Read overlay entry: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Read overlay file type: {e}"))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "Overlay symlinks are not allowed: {}",
                    entry.path().display()
                ));
            }
            let rel = entry
                .path()
                .strip_prefix(staging_dir)
                .map_err(|_| "Overlay path escaped staging".to_string())?
                .to_path_buf();
            let dest = safe_overlay_destination(game_dir, &rel, file_type.is_dir())?;
            if file_type.is_dir() {
                fs::create_dir_all(&dest).map_err(|e| format!("Create overlay dir: {e}"))?;
                pending.push(entry.path());
                continue;
            }
            if !file_type.is_file() {
                return Err(format!(
                    "Unsupported overlay entry: {}",
                    entry.path().display()
                ));
            }
            replace_from_staging(&entry.path(), &dest)?;
        }
    }
    Ok(())
}

fn safe_overlay_destination(game_dir: &Path, rel: &Path, expect_dir: bool) -> Result<PathBuf, String> {
    let rel = safe_rel(&rel.to_string_lossy())?;
    let mut current = game_dir.to_path_buf();
    if let Some(parent) = rel.parent() {
        for component in parent.components() {
            let Component::Normal(part) = component else {
                return Err("Unsafe overlay path".into());
            };
            current.push(part);
            match fs::symlink_metadata(&current) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    return Err(format!(
                        "Overlay destination contains a symlink: {}",
                        current.display()
                    ));
                }
                Ok(meta) if !meta.is_dir() => {
                    return Err(format!(
                        "Overlay destination parent is not a directory: {}",
                        current.display()
                    ));
                }
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current)
                        .map_err(|e| format!("Create overlay destination: {e}"))?;
                }
                Err(err) => return Err(format!("Inspect overlay destination: {err}")),
            }
        }
    }
    let dest = game_dir.join(rel);
    if let Ok(meta) = fs::symlink_metadata(&dest) {
        if meta.file_type().is_symlink() {
            return Err(format!("Unsafe overlay destination: {}", dest.display()));
        }
        if meta.is_dir() {
            // Vanilla/loader install already creates mods/, config/, etc.
            // A directory zip entry may merge into those folders.
            if expect_dir {
                return Ok(dest);
            }
            return Err(format!("Unsafe overlay destination: {}", dest.display()));
        }
        if expect_dir {
            return Err(format!(
                "Overlay directory target is a file: {}",
                dest.display()
            ));
        }
    }
    Ok(dest)
}

fn replace_from_staging(source: &Path, dest: &Path) -> Result<(), String> {
    let parent = dest
        .parent()
        .ok_or_else(|| "Overlay destination has no parent".to_string())?;
    let file_name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("overlay");
    let temp = parent.join(format!(".{file_name}.{}.owyx-tmp", std::process::id()));
    if temp.exists() {
        fs::remove_file(&temp).map_err(|e| format!("Clean overlay temp file: {e}"))?;
    }
    fs::copy(source, &temp).map_err(|e| format!("Copy overlay file: {e}"))?;
    File::open(&temp)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("Sync overlay file: {e}"))?;
    if let Err(first) = fs::rename(&temp, dest) {
        if dest.exists() {
            fs::remove_file(dest).map_err(|e| format!("Replace overlay file: {e}"))?;
            fs::rename(&temp, dest).map_err(|e| format!("Replace overlay file: {e}"))?;
        } else {
            let _ = fs::remove_file(&temp);
            return Err(format!("Move overlay file: {first}"));
        }
    }
    Ok(())
}

fn safe_rel(raw: &str) -> Result<PathBuf, String> {
    let cleaned = raw.replace('\\', "/");
    let mut out = PathBuf::new();
    for component in Path::new(&cleaned).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(format!("Unsafe zip path: {raw}")),
        }
    }
    if out.as_os_str().is_empty() {
        return Err("Empty zip path".into());
    }
    Ok(out)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn emit(
    app: &AppHandle,
    id: &str,
    phase: &str,
    done: u32,
    total: u32,
    message: String,
) -> Result<(), String> {
    app.emit(
        "install://progress",
        crate::models::SyncProgress {
            pack_id: id.to_string(),
            phase: phase.into(),
            current_file: None,
            done_files: done,
            total_files: total,
            message,
            file_bytes_done: None,
            file_bytes_total: None,
        },
    )
    .map_err(|e| format!("Emit progress: {e}"))
}

pub fn sanitize_server_addr(raw: &str) -> Result<String, String> {
    let s = raw.trim();
    if s.is_empty() || s.len() > 255 {
        return Err("Invalid server address".into());
    }
    if s.starts_with('-') || s.contains(char::is_whitespace) {
        return Err("Invalid server address".into());
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
    {
        return Err("Invalid server address".into());
    }
    Ok(s.to_string())
}

pub fn write_servers_dat(
    game_dir: &Path,
    name: &str,
    address: &str,
    port: u16,
) -> Result<(), String> {
    let address = sanitize_server_addr(address)?;
    let ip = if port == 25565 {
        address
    } else {
        format!("{address}:{port}")
    };
    let nbt = encode_servers_nbt(name, &ip);
    fs::create_dir_all(game_dir).map_err(|e| format!("Create game dir: {e}"))?;
    let path = game_dir.join("servers.dat");
    let file = File::create(&path).map_err(|e| format!("Create servers.dat: {e}"))?;
    let mut enc = GzEncoder::new(file, Compression::default());
    enc.write_all(&nbt)
        .map_err(|e| format!("Write servers.dat: {e}"))?;
    enc.finish().map_err(|e| format!("Gzip servers.dat: {e}"))?;
    Ok(())
}

fn encode_servers_nbt(name: &str, ip: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(10); // compound
    put_str(&mut out, "");
    // list "servers"
    out.push(9);
    put_str(&mut out, "servers");
    out.push(10); // element type = compound
    out.extend_from_slice(&1i32.to_be_bytes());
    put_named_string(&mut out, "name", name);
    put_named_string(&mut out, "ip", ip);
    out.push(1);
    put_str(&mut out, "hidden");
    out.push(0);
    out.push(0); // end server compound
    out.push(0); // end root
    out
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn put_named_string(out: &mut Vec<u8>, name: &str, value: &str) {
    out.push(8);
    put_str(out, name);
    put_str(out, value);
}

pub fn supports_quick_play(minecraft: &str) -> bool {
    let mut parts = minecraft.split('.');
    let major = parts
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let minor = parts
        .next()
        .and_then(|s| {
            s.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .ok()
        })
        .unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|s| {
            s.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .ok()
        })
        .unwrap_or(0);
    major > 1 || (major == 1 && (minor > 20 || (minor == 20 && patch >= 2)))
}

/// Skin / cosmetics download allowlist (same hosts as pack overlay).
pub fn assert_asset_url(raw: &str, api_base: &str) -> Result<(), String> {
    assert_pack_download_url(raw, api_base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_public_redirects_into_private_networks() {
        let public = reqwest::Url::parse("https://packs.example/pack.zip").unwrap();
        let private = reqwest::Url::parse("https://169.254.169.254/latest/meta-data").unwrap();
        assert!(!redirect_allowed(
            &public,
            &private,
            "http://127.0.0.1:3001"
        ));

        let local = reqwest::Url::parse("http://127.0.0.1:3001/pack.zip").unwrap();
        let local_next = reqwest::Url::parse("http://127.0.0.1:3001/files/pack.zip").unwrap();
        assert!(redirect_allowed(
            &local,
            &local_next,
            "http://127.0.0.1:3001"
        ));
    }

    #[test]
    fn blocks_https_downgrades_and_credentials() {
        let initial = reqwest::Url::parse("https://packs.example/pack.zip").unwrap();
        let downgrade = reqwest::Url::parse("http://packs.example/pack.zip").unwrap();
        let credentials = reqwest::Url::parse("https://user:pass@packs.example/pack.zip").unwrap();
        assert!(!redirect_allowed(
            &initial,
            &downgrade,
            "https://api.owyx.site"
        ));
        assert!(!redirect_allowed(
            &initial,
            &credentials,
            "https://api.owyx.site"
        ));
    }

    #[test]
    fn validates_overlay_paths_and_meta_inf() {
        assert_eq!(
            safe_rel(r"mods\example.jar").unwrap(),
            PathBuf::from("mods/example.jar")
        );
        assert!(safe_rel("../outside.txt").is_err());
        assert!(safe_rel("/absolute.txt").is_err());
        assert!(is_meta_inf_path("META-INF/MANIFEST.MF"));
        assert!(is_meta_inf_path("meta-inf/signature"));
        assert!(!is_meta_inf_path("mods/not-META-INF.jar"));
    }

    #[test]
    fn quick_play_starts_at_minecraft_1_20_2() {
        assert!(!supports_quick_play("1.19.4"));
        assert!(!supports_quick_play("1.20"));
        assert!(!supports_quick_play("1.20.1"));
        assert!(supports_quick_play("1.20.2"));
        assert!(supports_quick_play("1.21.1"));
    }

    #[test]
    fn failed_overlay_removes_ready_marker() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("owyx-catalog-test-{unique}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("meta.json"), b"{}").unwrap();
        fs::write(
            root.join("instance.json"),
            br#"{"id":"test","status":"ready"}"#,
        )
        .unwrap();

        mark_error_at(&root).unwrap();

        assert!(!root.join("meta.json").exists());
        let instance: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("instance.json")).unwrap()).unwrap();
        assert_eq!(instance["status"], "error");
        fs::remove_dir_all(root).unwrap();
    }
}
