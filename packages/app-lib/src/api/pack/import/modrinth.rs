//! Import Minecraft instances from the official Modrinth App data directory.
//!
//! Modrinth App (Theseus) stores game files under `profiles/<name>/`. Metadata
//! lives in SQLite; for import we treat each profile folder as a game directory
//! and detect loader/version from common markers when possible.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    State,
    install::{InstallPhaseDetails, InstallProgressReporter},
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
    util::io,
};

use super::{finish_import, recache_icon};

#[derive(Debug, Deserialize)]
struct ModrinthProfileMeta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    game_version: Option<String>,
    #[serde(default)]
    loader: Option<String>,
    #[serde(default)]
    loader_version: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    icon_path: Option<String>,
}

/// Default Modrinth App data roots (Windows Roaming / Linux share / macOS Application Support).
pub fn default_modrinth_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(data) = dirs::data_dir() {
        out.push(data.join("com.modrinth.theseus"));
        out.push(data.join("com.modrinth.ModrinthApp"));
        out.push(data.join("ModrinthApp"));
    }
    out
}

pub async fn is_valid_modrinth(instance_folder: PathBuf) -> bool {
    if !instance_folder.is_dir() {
        return false;
    }
    // Prefer explicit metadata from newer Modrinth App builds.
    if instance_folder.join("profile.json").exists()
        || instance_folder.join("modrinth_index.json").exists()
    {
        return true;
    }
    // Typical installed game tree.
    for marker in ["options.txt", "mods", "config", "resourcepacks", "saves"] {
        if instance_folder.join(marker).exists() {
            return true;
        }
    }
    false
}

fn detect_loader(folder: &Path) -> (ModLoader, Option<String>, Option<String>) {
    if let Ok(raw) = std::fs::read_to_string(folder.join("profile.json"))
        && let Ok(meta) = serde_json::from_str::<ModrinthProfileMeta>(&raw)
    {
        let loader = meta
            .loader
            .as_deref()
            .map(|s| match s.to_ascii_lowercase().as_str() {
                "fabric" => ModLoader::Fabric,
                "quilt" => ModLoader::Quilt,
                "forge" => ModLoader::Forge,
                "neoforge" => ModLoader::NeoForge,
                _ => ModLoader::Vanilla,
            })
            .unwrap_or(ModLoader::Vanilla);
        return (loader, meta.game_version, meta.loader_version);
    }

    // Heuristic: look for loader jars / markers in mods/
    let mods = folder.join("mods");
    if mods.is_dir()
        && let Ok(rd) = std::fs::read_dir(&mods)
    {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if name.contains("fabric-loader") || name.contains("fabric-api") {
                return (ModLoader::Fabric, None, None);
            }
            if name.contains("quilt-loader") {
                return (ModLoader::Quilt, None, None);
            }
            if name.contains("neoforge") {
                return (ModLoader::NeoForge, None, None);
            }
            if name.contains("forge") && !name.contains("neoforge") {
                return (ModLoader::Forge, None, None);
            }
        }
    }

    (ModLoader::Vanilla, None, None)
}

pub async fn import_modrinth(
    base_path: PathBuf,
    instance_folder: String,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let folder = base_path.join("profiles").join(&instance_folder);
    if !folder.is_dir() {
        // Allow selecting the profiles folder itself as base.
        let alt = base_path.join(&instance_folder);
        if alt.is_dir() {
            return Box::pin(import_modrinth_folder(
                alt,
                instance_folder,
                instance_id,
                reporter,
                details,
            ))
            .await;
        }
        return Err(crate::ErrorKind::InputError(format!(
            "Modrinth profile folder not found: {}",
            folder.display()
        ))
        .into());
    }

    import_modrinth_folder(folder, instance_folder, instance_id, reporter, details)
        .await
}

async fn import_modrinth_folder(
    folder: PathBuf,
    folder_name: String,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let meta = if let Ok(raw) =
        io::read_any_encoding_to_string(&folder.join("profile.json"))
            .await
            .map(|x| x.0)
    {
        serde_json::from_str::<ModrinthProfileMeta>(&raw).ok()
    } else {
        None
    };

    let (detected_loader, detected_gv, detected_lv) = detect_loader(&folder);
    let game_version = meta
        .as_ref()
        .and_then(|m| m.game_version.clone())
        .or(detected_gv)
        .unwrap_or_else(|| "1.20.1".to_string());
    let mod_loader = meta
        .as_ref()
        .and_then(|m| m.loader.as_deref())
        .map(|s| match s.to_ascii_lowercase().as_str() {
            "fabric" => ModLoader::Fabric,
            "quilt" => ModLoader::Quilt,
            "forge" => ModLoader::Forge,
            "neoforge" => ModLoader::NeoForge,
            _ => ModLoader::Vanilla,
        })
        .unwrap_or(detected_loader);
    let loader_version_hint = meta
        .as_ref()
        .and_then(|m| m.loader_version.clone())
        .or(detected_lv);

    let name = meta
        .as_ref()
        .and_then(|m| m.name.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| folder_name.clone());

    let icon_rel = meta
        .as_ref()
        .and_then(|m| m.icon.clone().or(m.icon_path.clone()));
    let icon = if let Some(rel) = icon_rel {
        let p = folder.join(rel);
        if p.exists() {
            recache_icon(p).await?
        } else {
            None
        }
    } else {
        None
    };

    let loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version_hint.as_deref(),
        )
        .await
        .ok()
        .flatten()
    } else {
        None
    };

    crate::api::instance::edit(
        instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            name: Some(name),
            icon_path: Some(icon.map(|x| x.to_string_lossy().to_string())),
            content_set_patch: Some(AppliedContentSetPatch {
                source_kind: None,
                game_version: Some(game_version),
                protocol_version: Some(None),
                loader: Some(mod_loader),
                loader_version: Some(loader_version.map(|x| x.id)),
            }),
            ..EditInstance::default()
        },
    )
    .await?;

    let state = State::get().await?;
    finish_import(
        instance_id,
        folder,
        &state.io_semaphore,
        reporter,
        details,
    )
    .await?;

    Ok(())
}
