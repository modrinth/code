//! Local skin library — Modrinth Skins.vue spirit without Mojang equip API.
//!
//! Stores custom PNGs under `{data}/skins/library/`, ships Steve as a default,
//! applies selected texture to `{data}/skins/` + optional instance `game/owyx/`.
//! Owyx account skins continue to use `owyx::apply_skin` (/me → disk).

use crate::config;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const STEVE_PNG: &[u8] = include_bytes!("../assets/steve.png");
const MAX_SKIN_BYTES: u64 = 512 * 1024;
const META_NAME: &str = "library.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinEntry {
    pub id: String,
    pub name: String,
    /// classic | slim
    pub model: String,
    /// default | custom | owyx
    pub source: String,
    /// Absolute path or empty for built-in defaults (texture served via data URL from bytes).
    #[serde(default)]
    pub path: String,
    /// data:image/png;base64,… for UI preview when path is empty / for defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_data_url: Option<String>,
    pub is_equipped: bool,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryFile {
    #[serde(default)]
    skins: Vec<StoredSkin>,
    #[serde(default)]
    active_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredSkin {
    id: String,
    name: String,
    model: String,
    file: String,
    #[serde(default)]
    created_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn skins_root() -> Result<PathBuf, String> {
    let root = config::ensure_layout()?.join("skins");
    fs::create_dir_all(&root).map_err(|e| format!("Create skins dir: {e}"))?;
    Ok(root)
}

fn library_dir() -> Result<PathBuf, String> {
    let dir = skins_root()?.join("library");
    fs::create_dir_all(&dir).map_err(|e| format!("Create skins library: {e}"))?;
    Ok(dir)
}

fn meta_path() -> Result<PathBuf, String> {
    Ok(library_dir()?.join(META_NAME))
}

fn load_library() -> Result<LibraryFile, String> {
    let path = meta_path()?;
    if !path.exists() {
        return Ok(LibraryFile::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Read skin library: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid skin library: {e}"))
}

fn save_library(file: &LibraryFile) -> Result<(), String> {
    let path = meta_path()?;
    let raw = serde_json::to_string_pretty(file).map_err(|e| format!("Serialize skins: {e}"))?;
    config::atomic_write_path(&path, raw.as_bytes())
}

fn validate_png(bytes: &[u8]) -> Result<(), String> {
    // PNG signature (8) + IHDR length(4) + type(4) + data(13) + crc(4) = 33
    if bytes.len() < 33 || bytes.len() as u64 > MAX_SKIN_BYTES {
        return Err("Skin file size is not a valid PNG skin".into());
    }
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes[..8] != SIG {
        return Err("Skin is not a PNG".into());
    }
    let ihdr_len = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if &bytes[12..16] != b"IHDR" || ihdr_len != 13 {
        return Err("Skin PNG missing IHDR".into());
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    // Classic 64×32 or modern 64×64 skins only.
    if !((width == 64 && height == 64) || (width == 64 && height == 32)) {
        return Err(format!(
            "Skin must be 64×64 or 64×32 PNG (got {width}×{height})"
        ));
    }
    Ok(())
}

fn safe_library_filename(raw: &str) -> Result<String, String> {
    if raw.contains("..") || raw.contains('/') || raw.contains('\\') {
        return Err("Unsafe skin library file name".into());
    }
    let base = Path::new(raw)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if base.is_empty() || base == "." || base == ".." || base != raw {
        return Err("Invalid skin library file name".into());
    }
    if !base.ends_with(".png") {
        return Err("Skin library file must be .png".into());
    }
    if !base
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("Unsafe skin library file name".into());
    }
    Ok(base.to_string())
}

fn png_data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", b64_encode(bytes))
}

fn b64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
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

fn sanitize_skin_id(raw: &str) -> Result<String, String> {
    let id: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if id.is_empty() {
        return Err("Invalid skin id".into());
    }
    Ok(id)
}

fn detect_model(bytes: &[u8]) -> String {
    // Heuristic: arm column alpha at classic slim probe — default classic if unsure.
    // Full canvas decode needs image crate; keep classic unless filename hints slim/alex.
    let _ = bytes;
    "classic".into()
}

fn default_entries(active_id: Option<&str>) -> Vec<SkinEntry> {
    let steve_url = png_data_url(STEVE_PNG);
    let active = active_id.unwrap_or("default:steve");
    vec![SkinEntry {
        id: "default:steve".into(),
        name: "Steve".into(),
        model: "classic".into(),
        source: "default".into(),
        path: String::new(),
        texture_data_url: Some(steve_url),
        is_equipped: active == "default:steve",
        created_at: 0,
    }]
}

fn bytes_for_skin(lib: &LibraryFile, id: &str) -> Result<Vec<u8>, String> {
    if id == "default:steve" {
        return Ok(STEVE_PNG.to_vec());
    }
    let stored = lib
        .skins
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Unknown skin: {id}"))?;
    let file = safe_library_filename(&stored.file)?;
    let path = library_dir()?.join(file);
    let bytes = fs::read(&path).map_err(|e| format!("Read skin: {e}"))?;
    validate_png(&bytes)?;
    Ok(bytes)
}

fn write_equipped(bytes: &[u8], model: &str, instance_id: Option<&str>) -> Result<String, String> {
    let root = skins_root()?;
    let dest = root.join("active.png");
    config::atomic_write_path(&dest, bytes)?;
    config::atomic_write_path(root.join("active-model.txt").as_path(), model.as_bytes())?;

    if let Some(id) = instance_id.map(str::trim).filter(|s| !s.is_empty()) {
        crate::instances::validate_instance_id(id)?;
        let inst_dir = config::ensure_layout()?
            .join("instances")
            .join(id)
            .join("game")
            .join("owyx");
        fs::create_dir_all(&inst_dir).map_err(|e| format!("Create instance skin dir: {e}"))?;
        config::atomic_write_path(&inst_dir.join("skin.png"), bytes)?;
        config::atomic_write_path(&inst_dir.join("skin-model.txt"), model.as_bytes())?;
    }
    Ok(dest.display().to_string())
}

/// List defaults + saved custom skins. Marks equipped from library meta.
pub fn list_skins() -> Result<Vec<SkinEntry>, String> {
    let lib = load_library()?;
    let active = lib.active_id.clone();
    let mut out = default_entries(active.as_deref());
    for s in &lib.skins {
        let Ok(file) = safe_library_filename(&s.file) else {
            continue;
        };
        let path = library_dir()?.join(&file);
        let texture = fs::read(&path).ok().and_then(|b| {
            validate_png(&b).ok()?;
            Some(png_data_url(&b))
        });
        out.push(SkinEntry {
            id: s.id.clone(),
            name: s.name.clone(),
            model: s.model.clone(),
            source: "custom".into(),
            path: path.display().to_string(),
            texture_data_url: texture,
            is_equipped: active.as_deref() == Some(s.id.as_str()),
            created_at: s.created_at,
        });
    }
    // If nothing marked equipped but active.png exists, leave defaults as-is.
    if out.iter().all(|e| !e.is_equipped) {
        if let Some(first) = out.first_mut() {
            first.is_equipped = true;
        }
    }
    Ok(out)
}

/// Copy a PNG into the library and return the new entry.
pub fn import_skin(source_path: String, name: Option<String>) -> Result<SkinEntry, String> {
    let src = PathBuf::from(source_path.trim());
    if !src.is_file() {
        return Err("Skin file not found".into());
    }
    let bytes = fs::read(&src).map_err(|e| format!("Read skin file: {e}"))?;
    validate_png(&bytes)?;
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("skin")
        .to_string();
    let display_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&stem)
        .chars()
        .take(48)
        .collect::<String>();
    let id = sanitize_skin_id(&format!("custom-{}", now_secs()))?;
    let file_name = format!("{id}.png");
    let dest = library_dir()?.join(&file_name);
    config::atomic_write_path(&dest, &bytes)?;

    let model = {
        let lower = display_name.to_ascii_lowercase();
        if lower.contains("alex") || lower.contains("slim") {
            "slim".into()
        } else {
            detect_model(&bytes)
        }
    };

    let mut lib = load_library()?;
    lib.skins.push(StoredSkin {
        id: id.clone(),
        name: display_name.clone(),
        model: model.clone(),
        file: file_name,
        created_at: now_secs(),
    });
    save_library(&lib)?;

    Ok(SkinEntry {
        id,
        name: display_name,
        model,
        source: "custom".into(),
        path: dest.display().to_string(),
        texture_data_url: Some(png_data_url(&bytes)),
        is_equipped: false,
        created_at: now_secs(),
    })
}

/// Apply a library (or default) skin to disk + optional instance overlay.
pub fn apply_skin(skin_id: String, instance_id: Option<String>) -> Result<SkinEntry, String> {
    let id = skin_id.trim();
    if id.is_empty() {
        return Err("Skin id required".into());
    }
    let mut lib = load_library()?;
    let bytes = bytes_for_skin(&lib, id)?;
    let model = if id == "default:steve" {
        "classic".into()
    } else {
        lib.skins
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.model.clone())
            .unwrap_or_else(|| detect_model(&bytes))
    };
    let path = write_equipped(&bytes, &model, instance_id.as_deref())?;
    lib.active_id = Some(id.to_string());
    save_library(&lib)?;

    Ok(SkinEntry {
        id: id.to_string(),
        name: if id == "default:steve" {
            "Steve".into()
        } else {
            lib.skins
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| id.to_string())
        },
        model,
        source: if id.starts_with("default:") {
            "default".into()
        } else {
            "custom".into()
        },
        path,
        texture_data_url: Some(png_data_url(&bytes)),
        is_equipped: true,
        created_at: 0,
    })
}

pub fn remove_skin(skin_id: String) -> Result<(), String> {
    let id = sanitize_skin_id(skin_id.trim())?;
    if id.starts_with("default") {
        return Err("Cannot remove default skins".into());
    }
    let mut lib = load_library()?;
    let Some(idx) = lib.skins.iter().position(|s| s.id == id) else {
        return Err("Skin not found".into());
    };
    let file = safe_library_filename(&lib.skins[idx].file)?;
    let was_active = lib.active_id.as_deref() == Some(id.as_str());
    lib.skins.remove(idx);
    if was_active {
        lib.active_id = Some("default:steve".into());
        // Keep disk in sync with meta — rewrite equipped texture to Steve.
        write_equipped(STEVE_PNG, "classic", None)?;
    }
    save_library(&lib)?;
    let path = library_dir()?.join(file);
    let _ = fs::remove_file(path);
    Ok(())
}

/// Ensure Steve default bytes exist under skins/ for non-Tauri tooling.
#[allow(dead_code)]
pub fn ensure_default_assets() -> Result<(), String> {
    let path = skins_root()?.join("defaults").join("steve.png");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create defaults: {e}"))?;
    }
    if !path.exists() {
        config::atomic_write_path(&path, STEVE_PNG)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steve_png_is_valid() {
        validate_png(STEVE_PNG).expect("bundled steve");
        assert!(png_data_url(STEVE_PNG).starts_with("data:image/png;base64,"));
    }

    #[test]
    fn rejects_non_png() {
        assert!(validate_png(b"not a png!!!!").is_err());
    }

    #[test]
    fn rejects_wrong_dimensions() {
        // Minimal IHDR with 32×32 — not a Minecraft skin size.
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&32u32.to_be_bytes());
        png.extend_from_slice(&32u32.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0]); // bit depth, RGBA, etc.
        png.extend_from_slice(&[0, 0, 0, 0]); // fake crc
        assert!(validate_png(&png).is_err());
    }

    #[test]
    fn safe_library_filename_blocks_escape() {
        assert!(safe_library_filename("../evil.png").is_err());
        assert!(safe_library_filename("ok-skin.png").is_ok());
    }

    #[test]
    fn remove_equipped_rewrites_active_png() {
        let _ = config::ensure_layout().expect("layout");
        let id = format!("custom-test-{}", now_secs());
        let file = format!("{id}.png");
        let dest = library_dir().unwrap().join(&file);
        config::atomic_write_path(&dest, STEVE_PNG).unwrap();
        let mut lib = load_library().unwrap();
        lib.skins.push(StoredSkin {
            id: id.clone(),
            name: "Temp".into(),
            model: "classic".into(),
            file: file.clone(),
            created_at: now_secs(),
        });
        lib.active_id = Some(id.clone());
        save_library(&lib).unwrap();
        // Equip a non-steve texture marker by writing different bytes then applying steve copy as "custom"
        write_equipped(STEVE_PNG, "classic", None).unwrap();
        // Corrupt active.png to prove rewrite happens on remove.
        let active = skins_root().unwrap().join("active.png");
        config::atomic_write_path(&active, b"NOT-STEVE-ACTIVE-TEXTURE").unwrap();
        remove_skin(id).unwrap();
        let after = fs::read(&active).expect("active.png");
        assert_eq!(after, STEVE_PNG);
        let lib2 = load_library().unwrap();
        assert_eq!(lib2.active_id.as_deref(), Some("default:steve"));
    }
}
