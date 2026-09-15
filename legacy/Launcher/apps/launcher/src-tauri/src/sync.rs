use crate::api::UpdateClient;
use crate::config;
use crate::models::{PackLocalStatus, PackManifest, PackSummary, SyncProgress};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Emitter};

pub async fn list_packs(update_base_url: Option<String>) -> Result<Vec<PackSummary>, String> {
    let client = UpdateClient::from_config_url(update_base_url.as_deref())?;
    let response = client.fetch_packs().await?;
    Ok(response
        .packs
        .into_iter()
        .filter(|p| validate_pack_id(&p.id).is_ok())
        .collect())
}

pub fn pack_status(pack_id: &str) -> Result<PackLocalStatus, String> {
    validate_pack_id(pack_id)?;
    let root = instance_root(pack_id)?;
    let meta_path = root.join("meta.json");
    if !meta_path.exists() {
        return Ok(PackLocalStatus {
            pack_id: pack_id.to_string(),
            installed: false,
            version: None,
            ready: false,
        });
    }
    let raw = fs::read_to_string(&meta_path).map_err(|e| format!("Read meta.json: {e}"))?;
    let manifest: PackManifest =
        serde_json::from_str(&raw).map_err(|e| format!("Invalid meta.json: {e}"))?;

    let game_dir = root.join("game");
    // Libraries/assets live in shared meta/ after Theseus-style install — not only under game/.
    let ready = manifest.files.iter().all(|file| {
        resolve_manifest_path(&game_dir, &file.path)
            .ok()
            .map(|path| file_ready_on_disk(&path, file.size, &file.sha256))
            .unwrap_or(false)
    });

    Ok(PackLocalStatus {
        pack_id: pack_id.to_string(),
        installed: true,
        version: Some(manifest.version),
        ready,
    })
}

pub async fn sync_pack(
    app: AppHandle,
    pack_id: String,
    update_base_url: Option<String>,
) -> Result<PackLocalStatus, String> {
    validate_pack_id(&pack_id)?;
    let client = UpdateClient::from_config_url(update_base_url.as_deref())?;
    let packs = client.fetch_packs().await?;
    let summary = packs
        .packs
        .into_iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| format!("Pack not found: {pack_id}"))?;

    emit_progress(
        &app,
        SyncProgress {
            pack_id: pack_id.clone(),
            phase: "manifest".into(),
            current_file: None,
            done_files: 0,
            total_files: 0,
            message: format!("Загрузка манифеста · {}", client.base_url()),
            file_bytes_done: None,
            file_bytes_total: None,
        },
    )?;

    let manifest = client.fetch_manifest(&summary.manifest_url).await?;
    if manifest.id != pack_id {
        return Err(format!(
            "Manifest id mismatch: expected {pack_id}, got {}",
            manifest.id
        ));
    }

    let instance = instance_root(&pack_id)?;
    let game_dir = instance.join("game");
    let meta_path = instance.join("meta.json");

    // Mark install as not ready for the duration of mutation.
    if meta_path.exists() {
        let _ = fs::remove_file(&meta_path);
    }
    fs::create_dir_all(&game_dir).map_err(|e| format!("Create game dir: {e}"))?;

    let total = manifest.files.len() as u32;
    let mut done = 0u32;

    for file in &manifest.files {
        emit_progress(
            &app,
            SyncProgress {
                pack_id: pack_id.clone(),
                phase: "download".into(),
                current_file: Some(file.path.clone()),
                done_files: done,
                total_files: total,
                message: format!("Скачивание {}", file.path),
                file_bytes_done: None,
                file_bytes_total: None,
            },
        )?;

        let rel = safe_rel_path(&file.path)?;
        let dest = game_dir.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Create parent dir: {e}"))?;
        }

        if dest.exists() && file_matches_sha256(&dest, &file.sha256)? {
            done += 1;
            continue;
        }

        let max_bytes = file
            .size
            .map(|s| s.saturating_add(s / 10).saturating_add(1024))
            .unwrap_or(64 * 1024 * 1024)
            .min(512 * 1024 * 1024);
        let pack_id_ref = pack_id.as_str();
        let path_ref = file.path.as_str();
        let expected = file.size;
        let mut on_prog = |received: u64, content_len: Option<u64>| {
            emit_progress(
                &app,
                SyncProgress {
                    pack_id: pack_id_ref.to_string(),
                    phase: "download".into(),
                    current_file: Some(path_ref.to_string()),
                    done_files: done,
                    total_files: total,
                    message: format!("Скачивание {path_ref}"),
                    file_bytes_done: Some(received),
                    file_bytes_total: expected.or(content_len),
                },
            )
        };
        let bytes = client
            .download_bytes_with_progress(&file.url, max_bytes, Some(&mut on_prog))
            .await?;
        let hash = hex_sha256(&bytes);
        if !hash.eq_ignore_ascii_case(&file.sha256) {
            return Err(format!(
                "SHA256 mismatch for {}: expected {}, got {}",
                file.path, file.sha256, hash
            ));
        }

        atomic_write_bytes(&dest, &bytes)?;
        done += 1;

        emit_progress(
            &app,
            SyncProgress {
                pack_id: pack_id.clone(),
                phase: "verify".into(),
                current_file: Some(file.path.clone()),
                done_files: done,
                total_files: total,
                message: format!("Проверен {}", file.path),
                file_bytes_done: None,
                file_bytes_total: None,
            },
        )?;
    }

    let meta_raw = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Serialize meta.json: {e}"))?;
    atomic_write_bytes(&meta_path, meta_raw.as_bytes())?;

    emit_progress(
        &app,
        SyncProgress {
            pack_id: pack_id.clone(),
            phase: "done".into(),
            current_file: None,
            done_files: done,
            total_files: total,
            message: "Сборка готова".into(),
            file_bytes_done: None,
            file_bytes_total: None,
        },
    )?;

    pack_status(&pack_id)
}

fn instance_root(pack_id: &str) -> Result<PathBuf, String> {
    validate_pack_id(pack_id)?;
    let root = config::ensure_layout()?.join("instances").join(pack_id);
    fs::create_dir_all(&root).map_err(|e| format!("Create instance dir: {e}"))?;
    Ok(root)
}

fn validate_pack_id(pack_id: &str) -> Result<(), String> {
    crate::instances::validate_instance_id(pack_id).map_err(|_| "Invalid pack id".into())
}

fn safe_rel_path(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw);
    if path.is_absolute() {
        return Err(format!("Absolute paths are not allowed: {raw}"));
    }
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(format!("Unsafe path component in {raw}")),
        }
    }
    if out.as_os_str().is_empty() {
        return Err("Empty file path".into());
    }
    Ok(out)
}

fn resolve_manifest_path(game_dir: &Path, rel: &str) -> Result<PathBuf, String> {
    let preferred = config::resolve_game_or_meta(rel, game_dir)?;
    if preferred.is_file() {
        return Ok(preferred);
    }
    // Legacy installs kept libraries/assets under game/.
    let legacy = game_dir.join(safe_rel_path(rel)?);
    if legacy.is_file() {
        return Ok(legacy);
    }
    Ok(preferred)
}

/// Prefer size match when known (shared meta); fall back to SHA256 for older manifests.
fn file_ready_on_disk(path: &Path, expected_size: Option<u64>, sha256: &str) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    if let Some(sz) = expected_size {
        return meta.len() == sz;
    }
    file_matches_sha256(path, sha256).unwrap_or(false)
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn file_matches_sha256(path: &Path, expected: &str) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let bytes = fs::read(path).map_err(|e| format!("Read existing file: {e}"))?;
    Ok(hex_sha256(&bytes).eq_ignore_ascii_case(expected))
}

fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Destination has no parent".to_string())?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file"),
        std::process::id()
    ));
    {
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("Create temp file: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("Write temp file: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Sync temp file: {e}"))?;
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Replace file failed: {e}")
    })
}

fn emit_progress(app: &AppHandle, progress: SyncProgress) -> Result<(), String> {
    app.emit("sync://progress", progress)
        .map_err(|e| format!("Emit progress failed: {e}"))
}
