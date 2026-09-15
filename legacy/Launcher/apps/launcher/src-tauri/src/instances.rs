use crate::config;
use crate::install::InstanceRecord;
use crate::log as owyx_log;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInstance {
    pub id: String,
    pub name: String,
    pub loader: String,
    pub minecraft: String,
    pub loader_version: String,
    pub created_at: u64,
    pub status: String,
    pub ready: bool,
    pub path: String,
    pub has_icon: bool,
    /// From `{instance}/catalog.json` when present (survives localStorage wipe).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_pack_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

fn catalog_fields_for(id: &str) -> (Option<String>, Option<String>, Option<String>, Option<u16>) {
    match crate::catalog::read_catalog_meta(id) {
        Some(meta) => (
            Some(meta.server_id),
            meta.pack_id,
            Some(meta.address),
            Some(meta.port),
        ),
        None => (None, None, None, None),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntryInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_at: u64,
}

/// Folder name rules aligned with Modrinth Theseus `sanitize_instance_name`.
pub fn sanitize_instance_name(input: &str) -> String {
    let cleaned: String = input
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | '?' | '*' | ':' | '\'' | '"' | '|' | '<' | '>' | '!' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .take(64)
        .collect();
    let cleaned = cleaned
        .trim_matches(|c: char| c == '.' || c.is_whitespace())
        .to_string();
    if cleaned.is_empty() {
        "instance".into()
    } else {
        cleaned
    }
}

pub fn validate_instance_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 96 || id == "." || id == ".." {
        return Err("Invalid instance id".into());
    }
    if id.chars().any(|c| {
        matches!(
            c,
            '/' | '\\' | '?' | '*' | ':' | '"' | '|' | '<' | '>' | '\0'
        ) || c.is_control()
    }) {
        return Err("Invalid instance id".into());
    }
    // Single path segment only.
    if Path::new(id).components().count() != 1 {
        return Err("Invalid instance id".into());
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    validate_instance_id(id)
}

fn instance_root(id: &str) -> Result<PathBuf, String> {
    validate_id(id)?;
    Ok(config::ensure_layout()?.join("instances").join(id))
}

fn allocate_unique_id(base: &str) -> Result<String, String> {
    let base = sanitize_instance_name(base);
    validate_id(&base)?;
    let root = config::ensure_layout()?.join("instances");
    let mut candidate = base.clone();
    let mut n = 1u32;
    loop {
        let path = root.join(&candidate);
        if !path.exists() {
            validate_id(&candidate)?;
            return Ok(candidate);
        }
        candidate = format!("{base} ({n})");
        n += 1;
        if n > 999 {
            return Err("Could not allocate instance folder name".into());
        }
    }
}

/// Create instance folder + `instance.json` without downloading game files.
/// Full install runs later on Play (Modrinth-like: create first, install when needed).
pub fn prepare_instance(
    name: &str,
    loader: &str,
    minecraft: &str,
    loader_version: &str,
    icon_source: Option<&str>,
) -> Result<DiskInstance, String> {
    let loader = loader.trim().to_ascii_lowercase();
    if !matches!(
        loader.as_str(),
        "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
    ) {
        return Err(format!("Unsupported loader: {loader}"));
    }
    if minecraft.trim().is_empty() {
        return Err("Minecraft version is required".into());
    }
    if loader != "vanilla" && loader_version.trim().is_empty() {
        return Err("Loader version is required".into());
    }

    let display_name: String = name.trim().chars().take(48).collect();
    let display_name = if display_name.is_empty() {
        sanitize_instance_name(&format!(
            "{} {}",
            if loader == "vanilla" {
                "Vanilla"
            } else {
                &loader
            },
            minecraft.trim()
        ))
    } else {
        display_name
    };
    let id = allocate_unique_id(&display_name)?;
    let root = instance_root(&id)?;
    fs::create_dir_all(root.join("game").join("mods")).map_err(|e| format!("Create mods: {e}"))?;
    fs::create_dir_all(root.join("game").join("resourcepacks"))
        .map_err(|e| format!("Create resourcepacks: {e}"))?;
    fs::create_dir_all(root.join("game").join("shaderpacks"))
        .map_err(|e| format!("Create shaderpacks: {e}"))?;
    fs::create_dir_all(root.join("game").join("saves"))
        .map_err(|e| format!("Create saves: {e}"))?;

    let created_at = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let record = InstanceRecord {
        id: id.clone(),
        name: display_name.clone(),
        loader: loader.clone(),
        minecraft: minecraft.trim().to_string(),
        loader_version: loader_version.trim().to_string(),
        created_at,
        status: "pending".into(),
    };
    let raw = serde_json::to_string_pretty(&record).map_err(|e| format!("Serialize: {e}"))?;
    fs::write(root.join("instance.json"), raw).map_err(|e| format!("Write instance.json: {e}"))?;

    if let Some(src) = icon_source.map(str::trim).filter(|s| !s.is_empty()) {
        if let Err(e) = set_instance_icon(&id, Some(src)) {
            let _ = fs::remove_dir_all(&root);
            return Err(e);
        }
    }

    let (catalog_server_id, catalog_pack_id, server_address, server_port) =
        catalog_fields_for(&id);
    Ok(DiskInstance {
        id,
        name: display_name,
        loader,
        minecraft: minecraft.trim().to_string(),
        loader_version: loader_version.trim().to_string(),
        created_at,
        status: "pending".into(),
        ready: false,
        path: root.display().to_string(),
        has_icon: icon_dest(&root).is_file(),
        catalog_server_id,
        catalog_pack_id,
        server_address,
        server_port,
    })
}

fn icon_dest(root: &Path) -> PathBuf {
    // Prefer existing icon.* if present; default write target is icon.png.
    for name in ["icon.png", "icon.jpg", "icon.jpeg", "icon.webp", "icon.gif", "icon.bmp"] {
        let p = root.join(name);
        if p.is_file() {
            return p;
        }
    }
    root.join("icon.png")
}

fn icon_write_dest(root: &Path, source: &Path) -> PathBuf {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"))
        .unwrap_or_else(|| "png".into());
    root.join(format!("icon.{ext}"))
}

fn clear_instance_icons(root: &Path) -> Result<(), String> {
    for name in ["icon.png", "icon.jpg", "icon.jpeg", "icon.webp", "icon.gif", "icon.bmp"] {
        let p = root.join(name);
        if p.is_file() {
            fs::remove_file(&p).map_err(|e| format!("Remove icon: {e}"))?;
        }
    }
    Ok(())
}

fn clear_other_instance_icons(root: &Path, keep: &Path) -> Result<(), String> {
    for name in ["icon.png", "icon.jpg", "icon.jpeg", "icon.webp", "icon.gif", "icon.bmp"] {
        let p = root.join(name);
        if p == keep {
            continue;
        }
        if p.is_file() {
            fs::remove_file(&p).map_err(|e| format!("Remove old icon: {e}"))?;
        }
    }
    Ok(())
}

fn is_image_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
            )
        })
        .unwrap_or(false)
}

/// Copy a user-picked image into `{instance}/icon.<ext>`, or remove it when `source` is None.
pub fn set_instance_icon(id: &str, source: Option<&str>) -> Result<(), String> {
    let root = instance_root(id)?;
    match source.map(str::trim).filter(|s| !s.is_empty()) {
        None => clear_instance_icons(&root),
        Some(src) => {
            let src_path = Path::new(src);
            if !src_path.is_file() {
                return Err("Icon file not found".into());
            }
            if !is_image_ext(src_path) {
                return Err("Icon must be png, jpg, webp, gif, or bmp".into());
            }
            let meta = fs::metadata(src_path).map_err(|e| format!("Stat icon: {e}"))?;
            if meta.len() > 8 * 1024 * 1024 {
                return Err("Icon file is too large (max 8 MB)".into());
            }
            let dest = icon_write_dest(&root, src_path);
            // Write via temp first so a failed copy never deletes the previous icon.
            let tmp = root.join(format!(
                ".{}.tmp",
                dest.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("icon.png")
            ));
            fs::copy(src_path, &tmp).map_err(|e| format!("Copy icon: {e}"))?;
            // Atomic replace: std::fs::rename overwrites dest on Windows too
            // (MoveFileExW + MOVEFILE_REPLACE_EXISTING). Do not delete dest first —
            // a failed rename must leave the previous icon intact.
            fs::rename(&tmp, &dest).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                format!("Replace icon: {e}")
            })?;
            // Drop other formats only after the new icon is in place.
            clear_other_instance_icons(&root, &dest)?;
            Ok(())
        }
    }
}

/// Read instance icon (or arbitrary image path) as a data URL for the WebView.
pub fn image_as_data_url(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Err("Image not found".into());
    }
    let meta = fs::metadata(path).map_err(|e| format!("Stat image: {e}"))?;
    if meta.len() > 8 * 1024 * 1024 {
        return Err("Image is too large".into());
    }
    let bytes = fs::read(path).map_err(|e| format!("Read image: {e}"))?;
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    Ok(format!(
        "data:{mime};base64,{}",
        base64_encode(&bytes)
    ))
}

pub fn instance_icon_data_url(id: &str) -> Result<Option<String>, String> {
    let root = instance_root(id)?;
    let dest = icon_dest(&root);
    if !dest.is_file() {
        return Ok(None);
    }
    Ok(Some(image_as_data_url(&dest)?))
}

fn instance_has_icon(root: &Path) -> bool {
    icon_dest(root).is_file()
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn safe_rel(raw: &str) -> Result<PathBuf, String> {
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
    Ok(out)
}

fn path_under(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|e| format!("Resolve root: {e}"))?;
    let full = if candidate.exists() {
        fs::canonicalize(candidate).map_err(|e| format!("Resolve path: {e}"))?
    } else {
        return Err("Path does not exist".into());
    };
    if !full.starts_with(&root) {
        return Err("Path escapes instance root".into());
    }
    Ok(full)
}

pub fn list_instances() -> Result<Vec<DiskInstance>, String> {
    let root = config::ensure_layout()?.join("instances");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| format!("Read instances: {e}"))? {
        let entry = entry.map_err(|e| format!("Read instances entry: {e}"))?;
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if validate_id(&id).is_err() {
            continue;
        }
        let record_path = entry.path().join("instance.json");
        let mut record: InstanceRecord = if record_path.is_file() {
            let raw = fs::read_to_string(&record_path)
                .map_err(|e| format!("Read instance.json: {e}"))?;
            serde_json::from_str(&raw).unwrap_or(InstanceRecord {
                id: id.clone(),
                name: id.clone(),
                loader: "vanilla".into(),
                minecraft: "".into(),
                loader_version: "".into(),
                created_at: 0,
                status: "pending".into(),
            })
        } else {
            InstanceRecord {
                id: id.clone(),
                name: id.clone(),
                loader: "vanilla".into(),
                minecraft: "".into(),
                loader_version: "".into(),
                created_at: 0,
                status: "pending".into(),
            }
        };
        // Folder name is canonical — fixes partial duplicate copies that still carry the source id.
        if record.id != id {
            record.id = id.clone();
            if let Ok(raw) = serde_json::to_string_pretty(&record) {
                let _ = fs::write(&record_path, raw);
            }
        }
        let mut status = if record.status.is_empty() {
            if entry.path().join("meta.json").is_file() {
                "ready".into()
            } else {
                "pending".into()
            }
        } else {
            record.status.clone()
        };
        // Crash/restart left "installing" without a live lock — treat as pending and heal JSON.
        if status == "installing" && !crate::install::is_install_active(&id) {
            status = "pending".into();
            record.status = "pending".into();
            if let Ok(raw) = serde_json::to_string_pretty(&record) {
                let _ = fs::write(&record_path, raw);
            }
        }
        // Cheap listing signal only — full verify stays on Play/repair (shared meta aware).
        let ready = entry.path().join("meta.json").is_file() && status == "ready";
        let (catalog_server_id, catalog_pack_id, server_address, server_port) =
            catalog_fields_for(&id);
        out.push(DiskInstance {
            id,
            name: record.name,
            loader: record.loader,
            minecraft: record.minecraft,
            loader_version: record.loader_version,
            created_at: record.created_at,
            status,
            ready,
            path: entry.path().display().to_string(),
            has_icon: instance_has_icon(&entry.path()),
            catalog_server_id,
            catalog_pack_id,
            server_address,
            server_port,
        });
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.name.cmp(&b.name)));
    Ok(out)
}

pub fn delete_instance(id: &str) -> Result<(), String> {
    let root = instance_root(id)?;
    if !root.exists() {
        return Ok(());
    }
    // Only block while an install lock is held — stale "installing" in JSON must not trap delete.
    if crate::install::is_install_active(id) {
        return Err("Cannot delete instance while install is in progress".into());
    }
    fs::remove_dir_all(&root).map_err(|e| format!("Delete instance: {e}"))
}

pub fn open_instance_dir(app: AppHandle, id: &str, which: &str) -> Result<(), String> {
    let target = match which {
        "logs" => owyx_log::logs_dir()?,
        "root" => {
            validate_id(id)?;
            instance_root(id)?
        }
        "game" => {
            validate_id(id)?;
            instance_root(id)?.join("game")
        }
        "mods" => {
            validate_id(id)?;
            instance_root(id)?.join("game").join("mods")
        }
        "saves" => {
            validate_id(id)?;
            instance_root(id)?.join("game").join("saves")
        }
        _ => return Err(format!("Unknown folder kind: {which}")),
    };
    if !target.exists() {
        fs::create_dir_all(&target).map_err(|e| format!("Create folder: {e}"))?;
    }
    if which != "logs" {
        let root = instance_root(id)?;
        let _ = path_under(&root, &target)?;
    }
    let canon = fs::canonicalize(&target).map_err(|e| format!("Resolve folder: {e}"))?;
    app.opener()
        .open_path(canon.display().to_string(), None::<&str>)
        .map_err(|e| format!("Open folder: {e}"))
}

pub fn list_instance_dir(id: &str, relative: &str) -> Result<Vec<DirEntryInfo>, String> {
    let root = instance_root(id)?.join("game");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let rel = if relative.trim().is_empty() {
        PathBuf::new()
    } else {
        safe_rel(relative)?
    };
    let dir = if rel.as_os_str().is_empty() {
        root.clone()
    } else {
        root.join(&rel)
    };
    let dir = path_under(&root, &dir)?;
    if !dir.is_dir() {
        return Err("Not a directory".into());
    }
    let mut out = Vec::new();
    // Cap listing so huge folders (assets/objects) cannot freeze the UI.
    const MAX_ENTRIES: usize = 2_000;
    for entry in fs::read_dir(&dir).map_err(|e| format!("Read dir: {e}"))? {
        if out.len() >= MAX_ENTRIES {
            break;
        }
        let entry = entry.map_err(|e| format!("Read entry: {e}"))?;
        // Prefer file_type() — cheaper than full metadata() on Windows.
        let file_type = entry
            .file_type()
            .map_err(|e| format!("File type: {e}"))?;
        let is_dir = file_type.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();
        let child_rel = if rel.as_os_str().is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel.to_string_lossy().replace('\\', "/"), name)
        };
        let size = if is_dir {
            None
        } else {
            entry.metadata().ok().map(|m| m.len())
        };
        out.push(DirEntryInfo {
            name,
            path: child_rel.replace('\\', "/"),
            is_dir,
            size,
        });
    }
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
    });
    Ok(out)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceLogInfo {
    pub id: String,
    pub name: String,
    pub source: String,
    pub filename: String,
    pub size: u64,
    pub modified_at: u64,
    pub live: bool,
}

/// Live Log + game/logs (+ .gz) + Owyx launcher launch-*.log for this instance.
pub fn list_instance_logs(pack_id: &str) -> Result<Vec<InstanceLogInfo>, String> {
    validate_instance_id(pack_id)?;
    let mut out = Vec::new();
    out.push(InstanceLogInfo {
        id: "live".into(),
        name: "Live Log".into(),
        source: "live".into(),
        filename: String::new(),
        size: 0,
        modified_at: 0,
        live: true,
    });

    // Minecraft game logs
    let game_logs = instance_root(pack_id)?.join("game").join("logs");
    if game_logs.is_dir() {
        for entry in fs::read_dir(&game_logs).map_err(|e| format!("Read game logs: {e}"))? {
            let entry = entry.map_err(|e| format!("Read game log entry: {e}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if !(lower.ends_with(".log") || lower.ends_with(".log.gz") || lower.ends_with(".txt")) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            out.push(InstanceLogInfo {
                id: format!("game:{name}"),
                name: name.clone(),
                source: "game".into(),
                filename: name,
                size: meta.len(),
                modified_at,
                live: false,
            });
        }
    }

    // Owyx launcher launch logs for this pack
    let launcher_dir = owyx_log::logs_dir()?;
    if launcher_dir.is_dir() {
        for entry in fs::read_dir(&launcher_dir).map_err(|e| format!("Read logs: {e}"))? {
            let entry = entry.map_err(|e| format!("Read log entry: {e}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".log") {
                continue;
            }
            if !is_launch_log_for_pack(&name, pack_id) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            out.push(InstanceLogInfo {
                id: format!("launcher:{name}"),
                name: format!("launcher/{name}"),
                source: "launcher".into(),
                filename: name,
                size: meta.len(),
                modified_at,
                live: false,
            });
        }
    }

    // Live first, then newest historical.
    let (live, mut rest): (Vec<_>, Vec<_>) = out.into_iter().partition(|l| l.live);
    rest.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    let mut merged = live;
    merged.extend(rest);
    Ok(merged)
}

pub fn read_instance_log(
    pack_id: &str,
    source: &str,
    filename: &str,
    max_bytes: Option<u64>,
) -> Result<String, String> {
    validate_instance_id(pack_id)?;
    let limit = max_bytes.unwrap_or(512 * 1024).clamp(4 * 1024, 2 * 1024 * 1024);

    let path = match source {
        "live" => resolve_live_log_path(pack_id)?,
        "game" => {
            let safe = Path::new(filename)
                .file_name()
                .ok_or_else(|| "Invalid log name".to_string())?;
            let path = instance_root(pack_id)?.join("game").join("logs").join(safe);
            if !path.is_file() {
                return Err("Log not found".into());
            }
            path
        }
        "launcher" => {
            let safe = Path::new(filename)
                .file_name()
                .ok_or_else(|| "Invalid log name".to_string())?;
            let name = safe.to_string_lossy();
            if !is_launch_log_for_pack(&name, pack_id) {
                return Err("Log does not belong to this instance".into());
            }
            let path = owyx_log::logs_dir()?.join(safe);
            if !path.is_file() {
                return Err("Log not found".into());
            }
            path
        }
        _ => return Err("Unknown log source".into()),
    };

    read_log_file_tail(&path, limit)
}

/// Match only `launch-{pack_id}-{unix_secs}.log` (same shape as `create_launch_log`).
/// Require the stamp to be digits so `mod` cannot claim `launch-mod-pack-….log`.
fn is_launch_log_for_pack(name: &str, pack_id: &str) -> bool {
    let Some(rest) = name.strip_prefix(&format!("launch-{pack_id}-")) else {
        return false;
    };
    let Some(stamp) = rest.strip_suffix(".log") else {
        return false;
    };
    !stamp.is_empty() && stamp.chars().all(|c| c.is_ascii_digit())
}

fn resolve_live_log_path(pack_id: &str) -> Result<PathBuf, String> {
    if let Some(path) = crate::launch::live_log_path(pack_id) {
        if path.is_file() {
            return Ok(path);
        }
    }
    // Prefer Minecraft latest.log while/after a session.
    let latest = instance_root(pack_id)?.join("game").join("logs").join("latest.log");
    if latest.is_file() {
        return Ok(latest);
    }
    // Fall back to newest Owyx launch-*.log for this pack.
    let dir = owyx_log::logs_dir()?;
    let mut best: Option<(u64, PathBuf)> = None;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".log") {
                continue;
            }
            if !is_launch_log_for_pack(&name, pack_id) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if best.as_ref().map(|(m, _)| modified >= *m).unwrap_or(true) {
                best = Some((modified, entry.path()));
            }
        }
    }
    best.map(|(_, p)| p)
        .ok_or_else(|| "No live log yet — start the instance".into())
}

fn read_log_file_tail(path: &Path, limit: u64) -> Result<String, String> {
    use std::io::{Read, Seek, SeekFrom};

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name.ends_with(".gz") {
        // Stream-decompress and keep only the last `limit` bytes (never buffer the whole file).
        const MAX_COMPRESSED: u64 = 16 * 1024 * 1024;
        let meta_len = fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        if meta_len > MAX_COMPRESSED {
            return Err(format!(
                "Compressed log is too large ({} MB). Open it from the logs folder.",
                meta_len / (1024 * 1024)
            ));
        }
        let file = fs::File::open(path).map_err(|e| format!("Open log: {e}"))?;
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut chunks: Vec<Vec<u8>> = Vec::new();
        let mut total = 0u64;
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = decoder
                .read(&mut buf)
                .map_err(|e| format!("Decompress log: {e}"))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            chunks.push(buf[..n].to_vec());
            while total > limit && !chunks.is_empty() {
                let first_len = chunks[0].len() as u64;
                if total - first_len >= limit {
                    total -= first_len;
                    chunks.remove(0);
                } else {
                    let drop = (total - limit) as usize;
                    chunks[0].drain(..drop);
                    total = limit;
                    break;
                }
            }
        }
        let mut out = Vec::with_capacity(total as usize);
        for chunk in chunks {
            out.extend_from_slice(&chunk);
        }
        return Ok(String::from_utf8_lossy(&out).into_owned());
    }

    let mut file = fs::File::open(path).map_err(|e| format!("Open log: {e}"))?;
    let len = file
        .metadata()
        .map_err(|e| format!("Stat log: {e}"))?
        .len();
    if len > limit {
        file.seek(SeekFrom::Start(len - limit))
            .map_err(|e| format!("Seek log: {e}"))?;
    }
    let mut buf = Vec::new();
    file.take(limit)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read log: {e}"))?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

pub fn list_launch_logs(pack_id: Option<String>) -> Result<Vec<LogFileInfo>, String> {
    let dir = owyx_log::logs_dir()?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("Read logs: {e}"))? {
        let entry = entry.map_err(|e| format!("Read log entry: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".log") {
            continue;
        }
        if let Some(pid) = pack_id.as_deref() {
            if !is_launch_log_for_pack(&name, pid) {
                continue;
            }
        }
        let meta = entry.metadata().map_err(|e| format!("Stat log: {e}"))?;
        if !meta.is_file() {
            continue;
        }
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push(LogFileInfo {
            name,
            path: entry.path().display().to_string(),
            size: meta.len(),
            modified_at,
        });
    }
    out.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(out)
}

pub fn read_launch_log(name: &str, max_bytes: Option<u64>) -> Result<String, String> {
    use std::io::{Read, Seek, SeekFrom};

    let safe = Path::new(name)
        .file_name()
        .ok_or_else(|| "Invalid log name".to_string())?;
    let path = owyx_log::logs_dir()?.join(safe);
    if !path.is_file() {
        return Err("Log not found".into());
    }
    let limit = max_bytes.unwrap_or(256 * 1024).clamp(4 * 1024, 2 * 1024 * 1024) as u64;
    let mut file = fs::File::open(&path).map_err(|e| format!("Open log: {e}"))?;
    let len = file
        .metadata()
        .map_err(|e| format!("Stat log: {e}"))?
        .len();
    if len > limit {
        file.seek(SeekFrom::Start(len - limit))
            .map_err(|e| format!("Seek log: {e}"))?;
    }
    let mut buf = Vec::new();
    file.take(limit)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read log: {e}"))?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub last_played_at: u64,
    pub hardcore: bool,
    /// `icon.png` of the world as a data URL (Theseus-style thumbnail), when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// World thumbnail: `icon.png` inside the save folder, as a data URL (small, so inline is fine).
fn world_icon_data_url(world_path: &Path) -> Option<String> {
    let icon = world_path.join("icon.png");
    if icon.is_file() {
        image_as_data_url(&icon).ok()
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_name: String,
    pub enabled: bool,
    pub size: u64,
    /// Modrinth project id when this file was installed via Modrinth API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modrinth_project_id: Option<String>,
}

fn content_dir(kind: &str) -> Result<&'static str, String> {
    match kind {
        "mod" => Ok("mods"),
        "resourcepack" => Ok("resourcepacks"),
        "shader" => Ok("shaderpacks"),
        "datapack" => Ok("datapacks"),
        _ => Err(format!("Unknown content kind: {kind}")),
    }
}

fn game_mode_label(game_type: i32, hardcore: bool) -> String {
    if hardcore {
        return "Hardcore".into();
    }
    match game_type {
        1 => "Creative".into(),
        2 => "Adventure".into(),
        3 => "Spectator".into(),
        _ => "Survival".into(),
    }
}

fn read_hardcore(data: &quartz_nbt::NbtCompound) -> bool {
    data.get::<_, &quartz_nbt::NbtCompound>("difficulty_settings")
        .ok()
        .and_then(|settings| settings.get::<_, i8>("hardcore").ok())
        .or_else(|| data.get::<_, i8>("hardcore").ok())
        .unwrap_or(0)
        != 0
}

/// Read LevelName / LastPlayed / GameType from `level.dat` (Modrinth Theseus pattern via quartz_nbt).
fn read_world_from_level_dat(world_path: &Path, folder_name: &str) -> WorldInfo {
    let fallback_modified = fs::metadata(world_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let icon = world_icon_data_url(world_path);
    let level_dat = world_path.join("level.dat");
    let Ok(raw) = fs::read(&level_dat) else {
        return WorldInfo {
            id: folder_name.into(),
            name: folder_name.into(),
            mode: "Local save".into(),
            last_played_at: fallback_modified,
            hardcore: false,
            icon,
        };
    };
    let parsed = quartz_nbt::io::read_nbt(
        &mut std::io::Cursor::new(raw),
        quartz_nbt::io::Flavor::GzCompressed,
    );
    let Ok((root, _)) = parsed else {
        return WorldInfo {
            id: folder_name.into(),
            name: folder_name.into(),
            mode: "Local save".into(),
            last_played_at: fallback_modified,
            hardcore: false,
            icon,
        };
    };
    let Ok(data) = root.get::<_, &quartz_nbt::NbtCompound>("Data") else {
        return WorldInfo {
            id: folder_name.into(),
            name: folder_name.into(),
            mode: "Local save".into(),
            last_played_at: fallback_modified,
            hardcore: false,
            icon,
        };
    };
    let level_name = data
        .get::<_, &str>("LevelName")
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| folder_name.to_string());
    let last_played = data
        .get::<_, i64>("LastPlayed")
        .ok()
        .filter(|v| *v > 0)
        .map(|v| v as u64)
        .unwrap_or(fallback_modified);
    let game_type = data.get::<_, i32>("GameType").unwrap_or(0);
    let hardcore = read_hardcore(data);
    WorldInfo {
        id: folder_name.into(),
        name: level_name,
        mode: game_mode_label(game_type, hardcore),
        last_played_at: last_played,
        hardcore,
        icon,
    }
}

pub fn list_worlds(id: &str) -> Result<Vec<WorldInfo>, String> {
    let saves = instance_root(id)?.join("game").join("saves");
    if !saves.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&saves).map_err(|e| format!("Read saves: {e}"))? {
        let entry = entry.map_err(|e| format!("Read save entry: {e}"))?;
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        // Prefer folders that look like real worlds (level.dat), same as Modrinth.
        if !entry.path().join("level.dat").is_file() {
            continue;
        }
        out.push(read_world_from_level_dat(&entry.path(), &name));
    }
    out.sort_by(|a, b| b.last_played_at.cmp(&a.last_played_at).then(a.name.cmp(&b.name)));
    Ok(out)
}

pub fn delete_world(id: &str, world_id: &str) -> Result<(), String> {
    let safe = Path::new(world_id)
        .file_name()
        .ok_or_else(|| "Invalid world id".to_string())?;
    let root = instance_root(id)?.join("game").join("saves");
    let target = root.join(safe);
    let target = path_under(&root, &target)?;
    if !target.is_dir() {
        return Err("World not found".into());
    }
    fs::remove_dir_all(&target).map_err(|e| format!("Delete world: {e}"))
}

pub fn list_content(id: &str) -> Result<Vec<ContentInfo>, String> {
    let game = instance_root(id)?.join("game");
    let modrinth_index = load_modrinth_index(id).unwrap_or_default();
    let mut out = Vec::new();
    for (kind, folder) in [
        ("mod", "mods"),
        ("resourcepack", "resourcepacks"),
        ("shader", "shaderpacks"),
        ("datapack", "datapacks"),
    ] {
        let dir = game.join(folder);
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|e| format!("Read {folder}: {e}"))? {
            let entry = entry.map_err(|e| format!("Read {folder} entry: {e}"))?;
            let meta = entry.metadata().map_err(|e| format!("Stat content: {e}"))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') {
                continue;
            }
            let enabled = !file_name.to_ascii_lowercase().ends_with(".disabled");
            let display = if enabled {
                file_name.clone()
            } else {
                file_name[..file_name.len().saturating_sub(".disabled".len())].to_string()
            };
            if kind == "mod" {
                let lower = display.to_ascii_lowercase();
                if !lower.ends_with(".jar") {
                    continue;
                }
            }
            let name = Path::new(&display)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| display.clone());
            let enabled_name = if file_name.to_ascii_lowercase().ends_with(".disabled") {
                file_name[..file_name.len().saturating_sub(".disabled".len())].to_string()
            } else {
                file_name.clone()
            };
            out.push(ContentInfo {
                id: format!("{kind}:{file_name}"),
                name,
                kind: kind.into(),
                file_name: file_name.clone(),
                enabled,
                size: if meta.is_file() { meta.len() } else { 0 },
                modrinth_project_id: modrinth_index
                    .get(&file_name)
                    .or_else(|| modrinth_index.get(&enabled_name))
                    .cloned(),
            });
        }
    }
    out.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then(a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
    });
    Ok(out)
}

pub fn find_content_by_file_name(id: &str, file_name: &str) -> Result<Option<ContentInfo>, String> {
    Ok(list_content(id)?
        .into_iter()
        .find(|item| {
            item.file_name.eq_ignore_ascii_case(file_name)
                || item
                    .file_name
                    .strip_suffix(".disabled")
                    .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
        }))
}

pub fn set_content_enabled(id: &str, kind: &str, file_name: &str, enabled: bool) -> Result<(), String> {
    let folder = content_dir(kind)?;
    let safe = Path::new(file_name)
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dir = instance_root(id)?.join("game").join(folder);
    let current = dir.join(safe);
    let current = path_under(&dir, &current)?;
    if !current.exists() {
        return Err("Content file not found".into());
    }
    let name = current
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid content name".to_string())?
        .to_string();
    let is_disabled = name.to_ascii_lowercase().ends_with(".disabled");
    if enabled == !is_disabled {
        return Ok(());
    }
    let target_name = if enabled {
        name.trim_end_matches(".disabled")
            .trim_end_matches(".DISABLED")
            .to_string()
    } else {
        format!("{name}.disabled")
    };
    let target = dir.join(&target_name);
    if target.exists() {
        return Err("Target content name already exists".into());
    }
    fs::rename(&current, &target).map_err(|e| format!("Rename content: {e}"))?;
    // Keep hide-installed accurate: if the index key was the pre-rename filename, move it.
    retarget_modrinth_index_key(id, &name, &target_name)?;
    Ok(())
}

pub fn delete_content(id: &str, kind: &str, file_name: &str) -> Result<(), String> {
    let folder = content_dir(kind)?;
    let safe = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string();
    let dir = instance_root(id)?.join("game").join(folder);
    // Clear sidecar BEFORE unlink so a clear failure leaves the file for retry.
    // Also runs when the file is already gone (stale index after partial failure).
    clear_modrinth_index_for_file(id, &safe)?;

    let candidate = dir.join(&safe);
    if !candidate.exists() {
        // Idempotent: index already cleaned; nothing on disk.
        return Ok(());
    }
    let target = path_under(&dir, &candidate)?;
    if target.is_dir() {
        fs::remove_dir_all(&target).map_err(|e| format!("Delete content dir: {e}"))?;
    } else if target.is_file() {
        fs::remove_file(&target).map_err(|e| format!("Delete content file: {e}"))?;
    }
    Ok(())
}

pub fn read_content_file(id: &str, kind: &str, file_name: &str) -> Result<Option<Vec<u8>>, String> {
    let target = content_target_path(id, kind, file_name)?;
    if !target.is_file() {
        return Ok(None);
    }
    fs::read(&target)
        .map(Some)
        .map_err(|e| format!("Read content: {e}"))
}

fn content_target_path(id: &str, kind: &str, file_name: &str) -> Result<std::path::PathBuf, String> {
    let folder = content_dir(kind)?;
    let safe = Path::new(file_name)
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dir = instance_root(id)?.join("game").join(folder);
    path_under(&dir, &dir.join(safe))
}

fn is_safe_file_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 200 {
        return false;
    }
    let path = Path::new(name);
    if path.components().count() != 1 {
        return false;
    }
    !matches!(
        path.components().next(),
        Some(Component::ParentDir | Component::RootDir | Component::Prefix(_))
    ) && !name.contains('/')
        && !name.contains('\\')
        && name != "."
        && name != ".."
}

/// Infer content kind from zip/jar markers (Modrinth `infer_project_type` logic).
fn infer_content_kind(bytes: &[u8]) -> Result<&'static str, String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|_| "Unable to infer content type for input file".to_string())?;
    if archive.by_name("fabric.mod.json").is_ok()
        || archive.by_name("quilt.mod.json").is_ok()
        || archive.by_name("META-INF/neoforge.mods.toml").is_ok()
        || archive.by_name("META-INF/mods.toml").is_ok()
        || archive.by_name("mcmod.info").is_ok()
    {
        return Ok("mod");
    }
    if archive.by_name("pack.mcmeta").is_ok() {
        let has_data = archive.file_names().any(|name| name.starts_with("data/"));
        return Ok(if has_data { "datapack" } else { "resourcepack" });
    }
    if archive
        .file_names()
        .any(|name| name.starts_with("shaders/"))
    {
        return Ok("shader");
    }
    Err("Unable to infer content type for input file".into())
}

const MAX_CONTENT_BYTES: u64 = 512 * 1024 * 1024;

/// Copy a local file into the instance content folder (Modrinth `add_project_from_path` pattern).
pub fn add_content_from_path(
    id: &str,
    path: &str,
    kind_hint: Option<&str>,
) -> Result<ContentInfo, String> {
    validate_id(id)?;
    let src = PathBuf::from(path);
    if !src.is_file() {
        return Err("Source file not found".into());
    }
    let file_name = src
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string();
    let meta = fs::metadata(&src).map_err(|e| format!("Stat source: {e}"))?;
    if meta.len() > MAX_CONTENT_BYTES {
        return Err("File too large".into());
    }
    let bytes = fs::read(&src).map_err(|e| format!("Read source: {e}"))?;
    add_content_from_bytes(id, &file_name, bytes, kind_hint)
}

/// Write already-fetched content bytes into the instance folder.
/// Shared by local-file add and Modrinth install; same safety (name, size, infer, path_under).
pub fn add_content_from_bytes(
    id: &str,
    file_name: &str,
    bytes: Vec<u8>,
    kind_hint: Option<&str>,
) -> Result<ContentInfo, String> {
    write_content_from_bytes(id, file_name, bytes, kind_hint, false)
}

/// Overwrite an existing jar atomically (temp + rename). Used for Modrinth
/// hash-mismatch upgrades so we never delete the old file before the new one
/// is on disk.
pub fn replace_content_from_bytes(
    id: &str,
    file_name: &str,
    bytes: Vec<u8>,
    kind_hint: Option<&str>,
) -> Result<ContentInfo, String> {
    write_content_from_bytes(id, file_name, bytes, kind_hint, true)
}

fn write_content_from_bytes(
    id: &str,
    file_name: &str,
    bytes: Vec<u8>,
    kind_hint: Option<&str>,
    overwrite: bool,
) -> Result<ContentInfo, String> {
    validate_id(id)?;
    if !is_safe_file_name(file_name) {
        return Err(format!("Invalid content file name: {file_name}"));
    }
    if bytes.len() as u64 > MAX_CONTENT_BYTES {
        return Err("File too large".into());
    }
    let kind = if let Some(hint) = kind_hint.filter(|k| !k.is_empty()) {
        content_dir(hint)?;
        hint
    } else {
        infer_content_kind(&bytes)?
    };
    let folder = content_dir(kind)?;
    let dir = instance_root(id)?.join("game").join(folder);
    fs::create_dir_all(&dir).map_err(|e| format!("Create {folder}: {e}"))?;
    // Canonicalize the content folder only — dest file does not exist yet.
    let dir = path_under(&dir, &dir)?;
    let dest = dir.join(file_name);
    if dest.exists() && !overwrite {
        return Err("Content file already exists".into());
    }
    config::atomic_write_path(&dest, &bytes)?;
    let name = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    Ok(ContentInfo {
        id: format!("{kind}:{file_name}"),
        name,
        kind: kind.into(),
        file_name: file_name.to_string(),
        enabled: true,
        size: bytes.len() as u64,
        modrinth_project_id: None,
    })
}

/// Sidecar index: file_name → Modrinth project id (for Hide installed / Installed badges).
fn modrinth_index_path(id: &str) -> Result<PathBuf, String> {
    Ok(instance_root(id)?.join("game").join(".owyx-modrinth-index.json"))
}

fn load_modrinth_index(id: &str) -> Result<HashMap<String, String>, String> {
    let path = modrinth_index_path(id)?;
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Read modrinth index: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse modrinth index: {e}"))
}

fn save_modrinth_index(id: &str, map: &HashMap<String, String>) -> Result<(), String> {
    let path = modrinth_index_path(id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create game dir: {e}"))?;
    }
    let raw = serde_json::to_vec_pretty(map).map_err(|e| format!("Serialize modrinth index: {e}"))?;
    config::atomic_write_path(&path, &raw)
}

/// Record that `file_name` was installed from Modrinth project `project_id`.
pub fn record_modrinth_project_id(id: &str, file_name: &str, project_id: &str) -> Result<(), String> {
    validate_id(id)?;
    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;
    let pid = project_id.trim();
    if pid.is_empty() || pid.len() > 64 {
        return Err("Invalid Modrinth project id".into());
    }
    let mut map = load_modrinth_index(id)?;
    map.insert(safe_name.to_string(), pid.to_string());
    save_modrinth_index(id, &map)
}

fn strip_disabled_suffix(name: &str) -> &str {
    let lower = name.to_ascii_lowercase();
    if let Some(stripped) = lower.strip_suffix(".disabled") {
        &name[..stripped.len()]
    } else {
        name
    }
}

/// Remove both enabled and `.disabled` keys so hide-installed cannot lie after delete/re-add.
fn clear_modrinth_index_for_file(id: &str, file_name: &str) -> Result<(), String> {
    let safe = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let base = strip_disabled_suffix(safe);
    let disabled = format!("{base}.disabled");
    let (mut map, load_ok) = match load_modrinth_index(id) {
        Ok(m) => (m, true),
        // Corrupt JSON → rewrite from empty rather than leave stale project ids.
        Err(_) => (HashMap::new(), false),
    };
    let mut changed = false;
    for key in [safe, base, disabled.as_str()] {
        if map.remove(key).is_some() {
            changed = true;
        }
    }
    if changed || !load_ok {
        save_modrinth_index(id, &map)?;
    }
    Ok(())
}

/// If the index used the pre-rename filename as key, move the project id to the new name.
fn retarget_modrinth_index_key(id: &str, from: &str, to: &str) -> Result<(), String> {
    if from == to {
        return Ok(());
    }
    let mut map = load_modrinth_index(id)?;
    if let Some(pid) = map.remove(from) {
        map.insert(to.to_string(), pid);
        save_modrinth_index(id, &map)?;
    }
    Ok(())
}

pub fn duplicate_instance(id: &str, new_id: &str, new_name: &str) -> Result<DiskInstance, String> {
    validate_id(id)?;
    validate_id(new_id)?;
    if id == new_id {
        return Err("Duplicate id must be different".into());
    }
    let src = instance_root(id)?;
    if !src.is_dir() {
        return Err("Source instance not found".into());
    }
    if crate::install::is_install_active(id) {
        return Err("Cannot duplicate while install is in progress".into());
    }
    let dst = instance_root(new_id)?;
    if dst.exists() {
        return Err("Target instance id already exists".into());
    }
    let result = (|| -> Result<DiskInstance, String> {
        copy_dir_recursive(&src, &dst)?;
        let mut record: InstanceRecord = if dst.join("instance.json").is_file() {
            let raw = fs::read_to_string(dst.join("instance.json"))
                .map_err(|e| format!("Read copied instance.json: {e}"))?;
            serde_json::from_str(&raw).map_err(|e| format!("Parse copied instance.json: {e}"))?
        } else {
            InstanceRecord {
                id: new_id.into(),
                name: new_name.into(),
                loader: "vanilla".into(),
                minecraft: "".into(),
                loader_version: "".into(),
                created_at: 0,
                status: "pending".into(),
            }
        };
        record.id = new_id.to_string();
        record.name = new_name.trim().chars().take(48).collect();
        if record.name.is_empty() {
            record.name = new_id.to_string();
        }
        record.created_at = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let raw = serde_json::to_string_pretty(&record).map_err(|e| format!("Serialize: {e}"))?;
        fs::write(dst.join("instance.json"), raw).map_err(|e| format!("Write instance.json: {e}"))?;

        let status = if record.status.is_empty() {
            if dst.join("meta.json").is_file() {
                "ready".into()
            } else {
                "pending".into()
            }
        } else {
            record.status.clone()
        };
        let (catalog_server_id, catalog_pack_id, server_address, server_port) =
            catalog_fields_for(&record.id);
        Ok(DiskInstance {
            id: record.id,
            name: record.name,
            loader: record.loader,
            minecraft: record.minecraft,
            loader_version: record.loader_version,
            created_at: record.created_at,
            ready: dst.join("meta.json").is_file() && status == "ready",
            status,
            path: dst.display().to_string(),
            has_icon: instance_has_icon(&dst),
            catalog_server_id,
            catalog_pack_id,
            server_address,
            server_port,
        })
    })();
    if result.is_err() && dst.exists() {
        let _ = fs::remove_dir_all(&dst);
    }
    result
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Create dest: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Read source: {e}"))? {
        let entry = entry.map_err(|e| format!("Read source entry: {e}"))?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        let ft = entry.file_type().map_err(|e| format!("File type: {e}"))?;
        if ft.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ft.is_file() {
            fs::copy(&from, &to).map_err(|e| format!("Copy file: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_temp_instance(test_id: &str, body: impl FnOnce(&str) -> Result<(), String>) {
        let id = format!("_test_{test_id}_{}", std::process::id());
        let root = config::ensure_layout()
            .expect("layout")
            .join("instances")
            .join(&id);
        let game = root.join("game").join("mods");
        fs::create_dir_all(&game).expect("mkdir");
        let result = body(&id);
        let _ = fs::remove_dir_all(&root);
        result.expect("test body");
    }

    #[test]
    fn delete_missing_file_still_clears_index_keys() {
        with_temp_instance("del_missing", |id| {
            record_modrinth_project_id(id, "sodium.jar", "AANobbMI")?;
            // Also leave a .disabled key as if enable/disable retargeted.
            let mut map = load_modrinth_index(id)?;
            map.insert("sodium.jar.disabled".into(), "AANobbMI".into());
            save_modrinth_index(id, &map)?;

            // File never existed — must still clear both keys.
            delete_content(id, "mod", "sodium.jar")?;
            let after = load_modrinth_index(id)?;
            assert!(
                !after.contains_key("sodium.jar"),
                "enabled key must be gone: {after:?}"
            );
            assert!(
                !after.contains_key("sodium.jar.disabled"),
                "disabled key must be gone: {after:?}"
            );
            Ok(())
        });
    }

    #[test]
    fn delete_clears_index_before_and_after_file_removal() {
        with_temp_instance("del_file", |id| {
            let jar = instance_root(id)?.join("game").join("mods").join("lithium.jar");
            fs::write(&jar, b"fake").map_err(|e| e.to_string())?;
            record_modrinth_project_id(id, "lithium.jar", "gvQqB4Zr")?;

            delete_content(id, "mod", "lithium.jar")?;
            assert!(!jar.exists());
            let after = load_modrinth_index(id)?;
            assert!(!after.contains_key("lithium.jar"));
            assert!(!after.contains_key("lithium.jar.disabled"));

            // Idempotent retry after partial failure scenario.
            record_modrinth_project_id(id, "lithium.jar", "gvQqB4Zr")?;
            delete_content(id, "mod", "lithium.jar")?;
            let after2 = load_modrinth_index(id)?;
            assert!(!after2.contains_key("lithium.jar"));
            Ok(())
        });
    }
}
