use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

static PROFILES_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub nick: String,
    #[serde(default)]
    pub needs_auth: bool,
    /// Opaque auth payload (Microsoft `ms1:` base64 JSON or Owyx). Treat as secret;
    /// not OS-encrypted yet (DPAPI/keychain follow-up). Never surface plaintext in UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_blob: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesFile {
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// Once true, empty profiles list is intentional (do not revive from config.nick).
    #[serde(default)]
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesState {
    pub profiles: Vec<Profile>,
    pub active_profile_id: Option<String>,
}

fn profiles_path() -> Result<PathBuf, String> {
    Ok(config::app_data_dir()?.join("profiles.json"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn with_profiles_lock<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = PROFILES_LOCK
        .lock()
        .map_err(|_| "Profiles lock poisoned".to_string())?;
    f()
}

fn load_file() -> Result<ProfilesFile, String> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(ProfilesFile::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read profiles: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid profiles.json: {e}"))
}

fn save_file(file: &ProfilesFile) -> Result<(), String> {
    let path = profiles_path()?;
    let raw = serde_json::to_string_pretty(file)
        .map_err(|e| format!("Failed to serialize profiles: {e}"))?;
    config::atomic_write_path(&path, raw.as_bytes())
}

pub fn load_state() -> Result<ProfilesState, String> {
    with_profiles_lock(|| {
        let _ = config::ensure_layout()?;
        let mut file = load_file()?;
        let mut cfg = config::load()?;

        // One-shot migration: legacy nick → offline profile, only if never initialized.
        if !file.initialized && file.profiles.is_empty() && !cfg.nick.trim().is_empty() {
            let nick = cfg.nick.trim().to_string();
            let id = format!("prof_{}", now_secs());
            file.profiles.push(Profile {
                id: id.clone(),
                kind: "offline".into(),
                name: nick.clone(),
                nick,
                needs_auth: false,
                auth_blob: None,
                created_at: now_secs(),
            });
            file.initialized = true;
            cfg.active_profile_id = Some(id);
            save_file(&file)?;
            config::save(&cfg)?;
        } else if !file.initialized {
            file.initialized = true;
            let _ = save_file(&file);
        }

        if let Some(active) = cfg.active_profile_id.clone() {
            if !file.profiles.iter().any(|p| p.id == active) {
                cfg.active_profile_id = file.profiles.first().map(|p| p.id.clone());
                config::save(&cfg)?;
            }
        }

        Ok(ProfilesState {
            profiles: file.profiles,
            active_profile_id: cfg.active_profile_id,
        })
    })
}

pub fn save_state(
    profiles: Vec<Profile>,
    active_profile_id: Option<String>,
) -> Result<ProfilesState, String> {
    with_profiles_lock(|| {
        for p in &profiles {
            if !matches!(p.kind.as_str(), "offline" | "microsoft" | "owyx") {
                return Err(format!("Unknown profile kind: {}", p.kind));
            }
            if p.nick.chars().count() > 16 {
                return Err("Nick must be at most 16 characters".into());
            }
            if !p.nick.is_empty()
                && !p
                    .nick
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return Err("Nick may only contain letters, numbers, and _".into());
            }
        }

        let active = active_profile_id
            .filter(|id| profiles.iter().any(|p| &p.id == id))
            .or_else(|| profiles.first().map(|p| p.id.clone()));

        save_file(&ProfilesFile {
            profiles: profiles.clone(),
            initialized: true,
        })?;

        let mut cfg = config::load()?;
        cfg.active_profile_id = active.clone();
        if let Some(id) = &active {
            if let Some(p) = profiles.iter().find(|p| &p.id == id) {
                if p.kind == "offline" {
                    cfg.nick = p.nick.clone();
                }
            }
        } else {
            cfg.nick.clear();
            cfg.active_profile_id = None;
        }
        config::save(&cfg)?;

        Ok(ProfilesState {
            profiles,
            active_profile_id: active,
        })
    })
}

pub fn active_nick() -> Result<String, String> {
    let state = load_state()?;
    let nick = state
        .active_profile_id
        .as_ref()
        .and_then(|id| state.profiles.iter().find(|p| &p.id == id))
        .map(|p| p.nick.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Player".into());
    Ok(nick)
}
