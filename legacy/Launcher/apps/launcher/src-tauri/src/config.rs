use crate::java::JavaInstallation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

static CONFIG_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterConfig {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub last_check_at: Option<u64>,
}

fn default_channel() -> String {
    "stable".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherConfig {
    pub nick: String,
    #[serde(default)]
    pub update_base_url: Option<String>,
    #[serde(default)]
    pub java_installations: Vec<JavaInstallation>,
    /// Preferred java path per major version (8 / 17 / 21 / 25), Modrinth-style.
    #[serde(default)]
    pub java_versions: HashMap<String, String>,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub active_profile_id: Option<String>,
    #[serde(default)]
    pub updater: UpdaterConfig,
    #[serde(default = "default_memory_mb")]
    pub default_memory_mb: u32,
}

fn default_locale() -> String {
    "ru".into()
}

fn default_memory_mb() -> u32 {
    4096
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            nick: String::new(),
            update_base_url: None,
            java_installations: Vec::new(),
            java_versions: HashMap::new(),
            locale: default_locale(),
            active_profile_id: None,
            updater: UpdaterConfig::default(),
            default_memory_mb: default_memory_mb(),
        }
    }
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    // Player data lives in %USERPROFILE%\owyx (not AppData, not *Launcher*).
    let home = dirs::home_dir().ok_or_else(|| "Could not resolve user home directory".to_string())?;
    Ok(home.join("owyx"))
}

pub fn ensure_layout() -> Result<PathBuf, String> {
    let root = app_data_dir()?;
    for sub in [
        "instances",
        "cache",
        "logs",
        "skins",
        "meta",
        "meta/libraries",
        "meta/assets",
        "meta/assets/indexes",
        "meta/assets/objects",
        "meta/natives",
        "meta/versions",
        "meta/log_configs",
        "meta/java_versions",
    ] {
        fs::create_dir_all(root.join(sub))
            .map_err(|e| format!("Failed to create {sub}/: {e}"))?;
    }
    Ok(root)
}

/// Shared Minecraft artifacts (Modrinth Theseus-style `meta/`).
pub fn meta_dir() -> Result<PathBuf, String> {
    Ok(ensure_layout()?.join("meta"))
}

pub fn meta_libraries_dir() -> Result<PathBuf, String> {
    Ok(meta_dir()?.join("libraries"))
}

pub fn meta_assets_dir() -> Result<PathBuf, String> {
    Ok(meta_dir()?.join("assets"))
}

pub fn meta_natives_dir(version: &str) -> Result<PathBuf, String> {
    let version = sanitize_meta_segment(version, "natives version")?;
    let dir = meta_dir()?.join("natives").join(version);
    fs::create_dir_all(&dir).map_err(|e| format!("Create natives dir: {e}"))?;
    Ok(dir)
}

fn sanitize_meta_segment(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 64 || value == "." || value == ".." {
        return Err(format!("Invalid {label}"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(format!("Invalid {label} characters"));
    }
    Ok(value.to_string())
}

pub fn meta_versions_dir() -> Result<PathBuf, String> {
    Ok(meta_dir()?.join("versions"))
}

/// Resolve a manifest-relative path: libraries/assets/versions live in shared meta, rest under game/.
/// Matches Modrinth Theseus: `meta/libraries`, `meta/assets`, `meta/versions/{id}/`.
pub fn resolve_game_or_meta(rel: &str, game_dir: &Path) -> Result<PathBuf, String> {
    let safe = safe_rel_component(rel)?;
    let trimmed = safe.to_string_lossy().replace('\\', "/");
    if trimmed.starts_with("libraries/")
        || trimmed.starts_with("assets/")
        || trimmed.starts_with("versions/")
    {
        Ok(meta_dir()?.join(&safe))
    } else {
        Ok(game_dir.join(&safe))
    }
}

/// Theseus-style shared version folder: `meta/versions/{id}/`.
pub fn meta_version_dir(version_id: &str) -> Result<PathBuf, String> {
    let id = sanitize_meta_segment(version_id, "minecraft version")?;
    let dir = meta_versions_dir()?.join(id);
    fs::create_dir_all(&dir).map_err(|e| format!("Create version dir: {e}"))?;
    Ok(dir)
}

fn safe_rel_component(raw: &str) -> Result<PathBuf, String> {
    use std::path::{Component, Path};
    let path = Path::new(raw.trim_start_matches(['/', '\\']));
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
        return Err("Empty relative path".into());
    }
    Ok(out)
}

fn with_config_lock<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = CONFIG_LOCK
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    f()
}

pub fn load() -> Result<LauncherConfig, String> {
    with_config_lock(load_unlocked)
}

fn load_unlocked() -> Result<LauncherConfig, String> {
    let root = ensure_layout()?;
    let path = root.join("config.json");
    if !path.exists() {
        let cfg = LauncherConfig::default();
        save_unlocked(&cfg)?;
        return Ok(cfg);
    }

    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;
    match serde_json::from_str::<LauncherConfig>(&raw) {
        Ok(cfg) => Ok(cfg),
        Err(err) => {
            // Corrupt/partial file: quarantine and recover so the UI can keep working.
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = root.join(format!("config.corrupt.{stamp}.json"));
            let _ = fs::rename(&path, &backup);
            let cfg = LauncherConfig::default();
            save_unlocked(&cfg)?;
            eprintln!("config.json invalid ({err}); backed up to {}", backup.display());
            Ok(cfg)
        }
    }
}

#[allow(dead_code)] // used by upcoming sync/settings commands
pub fn save(config: &LauncherConfig) -> Result<(), String> {
    with_config_lock(|| save_unlocked(config))
}

fn save_unlocked(config: &LauncherConfig) -> Result<(), String> {
    let root = ensure_layout()?;
    let path = root.join("config.json");
    let raw = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    atomic_write(&path, raw.as_bytes())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    atomic_write_path(path, bytes)
}

pub fn atomic_write_path(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Config path has no parent directory".to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("config.json"),
        std::process::id(),
        stamp
    ));

    {
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("Failed to create temp config: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("Failed to write temp config: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync temp config: {e}"))?;
    }

    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to replace config.json: {e}")
    })
}

pub fn set_nick(nick: String) -> Result<LauncherConfig, String> {
    let nick = nick.trim().to_string();
    if nick.chars().count() > 16 {
        return Err("Nick must be at most 16 characters".into());
    }
    if !nick.is_empty()
        && !nick
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Nick may only contain letters, numbers, and _".into());
    }

    with_config_lock(|| {
        let mut config = load_unlocked()?;
        config.nick = nick;
        save_unlocked(&config)?;
        Ok(config)
    })
}

pub fn set_java_installations(java: Vec<JavaInstallation>) -> Result<LauncherConfig, String> {
    with_config_lock(|| {
        let mut config = load_unlocked()?;
        config.java_installations = java;
        save_unlocked(&config)?;
        Ok(config)
    })
}

pub fn set_java_version_path(major: u32, path: Option<String>) -> Result<LauncherConfig, String> {
    with_config_lock(|| {
        let mut config = load_unlocked()?;
        let key = major.to_string();
        match path {
            Some(p) if !p.trim().is_empty() => {
                config.java_versions.insert(key, p.trim().to_string());
            }
            _ => {
                config.java_versions.remove(&key);
            }
        }
        save_unlocked(&config)?;
        Ok(config)
    })
}

pub fn java_versions_dir() -> Result<PathBuf, String> {
    Ok(meta_dir()?.join("java_versions"))
}

pub fn prefetch_marker_path() -> Result<PathBuf, String> {
    Ok(meta_dir()?.join(".prefetch_ready.json"))
}

pub fn set_locale(locale: String) -> Result<LauncherConfig, String> {
    let locale = if locale == "en" { "en" } else { "ru" }.to_string();
    with_config_lock(|| {
        let mut config = load_unlocked()?;
        config.locale = locale;
        save_unlocked(&config)?;
        Ok(config)
    })
}

pub fn set_updater_checked(channel: Option<String>) -> Result<LauncherConfig, String> {
    with_config_lock(|| {
        let mut config = load_unlocked()?;
        if let Some(ch) = channel {
            if !ch.trim().is_empty() {
                config.updater.channel = ch.trim().to_string();
            }
        }
        config.updater.last_check_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
        save_unlocked(&config)?;
        Ok(config)
    })
}
