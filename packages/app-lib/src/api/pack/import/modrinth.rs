//! Import Minecraft instances from the official Modrinth App data directory.
//!
//! Modrinth App (Theseus) stores game files under `profiles/<name>/`. Instance
//! metadata (game version, loader) lives in SQLite `app.db` next to `profiles/`.
//! Some folders also carry CurseForge leftovers (`minecraftinstance.json`,
//! `manifest.json`). Older builds may have `profile.json`.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Row};

use crate::{
    State,
    install::{InstallPhaseDetails, InstallProgressReporter},
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
};

use super::{finish_import, recache_icon};

#[derive(Debug, Clone, Default)]
struct DetectedMeta {
    name: Option<String>,
    game_version: Option<String>,
    loader: Option<ModLoader>,
    loader_version: Option<String>,
    icon: Option<String>,
}

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

#[derive(Debug, Deserialize)]
struct MinecraftInstanceFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default, rename = "gameVersion")]
    game_version: Option<String>,
    #[serde(default, rename = "baseModLoader")]
    base_mod_loader: Option<MinecraftInstanceLoader>,
}

#[derive(Debug, Deserialize)]
struct MinecraftInstanceLoader {
    #[serde(default)]
    name: Option<String>,
    #[serde(default, rename = "forgeVersion")]
    forge_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseManifest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    minecraft: Option<CurseManifestMinecraft>,
}

#[derive(Debug, Deserialize)]
struct CurseManifestMinecraft {
    #[serde(default)]
    version: Option<String>,
    #[serde(default, rename = "modLoaders")]
    mod_loaders: Vec<CurseManifestLoader>,
}

#[derive(Debug, Deserialize)]
struct CurseManifestLoader {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    primary: bool,
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
    // Prefer explicit metadata from newer Modrinth App builds / CF leftovers.
    if instance_folder.join("profile.json").exists()
        || instance_folder.join("modrinth_index.json").exists()
        || instance_folder.join("minecraftinstance.json").exists()
        || instance_folder.join("manifest.json").exists()
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

fn parse_loader_name(s: &str) -> ModLoader {
    match s.to_ascii_lowercase().as_str() {
        "fabric" => ModLoader::Fabric,
        "quilt" => ModLoader::Quilt,
        "forge" => ModLoader::Forge,
        "neoforge" => ModLoader::NeoForge,
        _ => ModLoader::Vanilla,
    }
}

fn parse_loader_id(id: &str) -> (ModLoader, Option<String>) {
    let lower = id.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("neoforge-") {
        return (ModLoader::NeoForge, Some(rest.to_string()));
    }
    if let Some(rest) = lower.strip_prefix("forge-") {
        return (ModLoader::Forge, Some(rest.to_string()));
    }
    if let Some(rest) = lower.strip_prefix("fabric-") {
        return (ModLoader::Fabric, Some(rest.to_string()));
    }
    if let Some(rest) = lower.strip_prefix("quilt-") {
        return (ModLoader::Quilt, Some(rest.to_string()));
    }
    (parse_loader_name(&lower), None)
}

fn find_modrinth_app_db(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    for _ in 0..6 {
        let Some(dir) = cur else { break };
        let candidate = dir.join("app.db");
        if candidate.is_file() {
            return Some(candidate);
        }
        // profiles/<instance> → check parent of profiles
        if dir.file_name().and_then(|s| s.to_str()) == Some("profiles") {
            let candidate = dir
                .parent()
                .map(|p| p.join("app.db"))
                .filter(|p| p.is_file());
            if candidate.is_some() {
                return candidate;
            }
        }
        cur = dir.parent();
    }
    None
}

async fn read_meta_from_app_db(
    db_path: &Path,
    instance_folder_name: &str,
) -> Option<DetectedMeta> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true)
        .disable_statement_logging();

    let mut conn = opts.connect().await.ok()?;

    // Prefer the applied content set (current install). Fall back to any row.
    let row = sqlx::query(
        r#"
        SELECT
            i.name AS name,
            i.icon_path AS icon_path,
            COALESCE(applied.game_version, any_cs.game_version) AS game_version,
            COALESCE(applied.loader, any_cs.loader) AS loader,
            COALESCE(applied.loader_version, any_cs.loader_version) AS loader_version
        FROM instances i
        LEFT JOIN instance_content_sets applied
            ON applied.id = i.applied_content_set_id
        LEFT JOIN instance_content_sets any_cs
            ON any_cs.instance_id = i.id
        WHERE i.path = ?1 OR i.path = ?2 OR i.name = ?1
        LIMIT 1
        "#,
    )
    .bind(instance_folder_name)
    .bind(format!("profiles/{instance_folder_name}"))
    .fetch_optional(&mut conn)
    .await
    .ok()??;

    let loader_str: Option<String> = row.try_get("loader").ok().flatten();
    Some(DetectedMeta {
        name: row.try_get("name").ok().flatten(),
        game_version: row.try_get("game_version").ok().flatten(),
        loader: loader_str.as_deref().map(parse_loader_name),
        loader_version: row.try_get("loader_version").ok().flatten(),
        icon: row.try_get("icon_path").ok().flatten(),
    })
}

fn read_meta_from_profile_json(folder: &Path) -> Option<DetectedMeta> {
    let raw = std::fs::read_to_string(folder.join("profile.json")).ok()?;
    let meta: ModrinthProfileMeta = serde_json::from_str(&raw).ok()?;
    Some(DetectedMeta {
        name: meta.name,
        game_version: meta.game_version,
        loader: meta.loader.as_deref().map(parse_loader_name),
        loader_version: meta.loader_version,
        icon: meta.icon.or(meta.icon_path),
    })
}

fn read_meta_from_minecraftinstance(folder: &Path) -> Option<DetectedMeta> {
    let raw =
        std::fs::read_to_string(folder.join("minecraftinstance.json")).ok()?;
    let meta: MinecraftInstanceFile = serde_json::from_str(&raw).ok()?;
    let (loader, loader_version) = meta
        .base_mod_loader
        .as_ref()
        .map(|l| {
            let from_name = l
                .name
                .as_deref()
                .map(parse_loader_id)
                .unwrap_or((ModLoader::Vanilla, None));
            let version = l.forge_version.clone().or(from_name.1);
            (from_name.0, version)
        })
        .unwrap_or((ModLoader::Vanilla, None));
    Some(DetectedMeta {
        name: meta.name,
        game_version: meta.game_version,
        loader: Some(loader),
        loader_version,
        icon: None,
    })
}

fn read_meta_from_manifest(folder: &Path) -> Option<DetectedMeta> {
    let raw = std::fs::read_to_string(folder.join("manifest.json")).ok()?;
    let meta: CurseManifest = serde_json::from_str(&raw).ok()?;
    let mc = meta.minecraft.as_ref()?;
    let primary = mc
        .mod_loaders
        .iter()
        .find(|l| l.primary)
        .or_else(|| mc.mod_loaders.first());
    let (loader, loader_version) = primary
        .and_then(|l| l.id.as_deref())
        .map(parse_loader_id)
        .unwrap_or((ModLoader::Vanilla, None));
    Some(DetectedMeta {
        name: meta.name,
        game_version: mc.version.clone(),
        loader: Some(loader),
        loader_version,
        icon: None,
    })
}

fn detect_loader_heuristic(
    folder: &Path,
) -> (ModLoader, Option<String>, Option<String>) {
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

fn merge_meta(into: &mut DetectedMeta, from: DetectedMeta) {
    if into.name.is_none() {
        into.name = from.name;
    }
    if into.game_version.is_none() {
        into.game_version = from.game_version;
    }
    if into.loader.is_none() {
        into.loader = from.loader;
    }
    if into.loader_version.is_none() {
        into.loader_version = from.loader_version;
    }
    if into.icon.is_none() {
        into.icon = from.icon;
    }
}

async fn resolve_meta(
    folder: &Path,
    folder_name: &str,
    search_root: &Path,
) -> DetectedMeta {
    let mut meta = DetectedMeta::default();

    // 1) Modrinth App SQLite — authoritative for Theseus installs.
    if let Some(db) = find_modrinth_app_db(search_root)
        .or_else(|| find_modrinth_app_db(folder))
        && let Some(from_db) = read_meta_from_app_db(&db, folder_name).await
    {
        merge_meta(&mut meta, from_db);
    }

    // 2) Optional on-disk metadata (CF leftovers / legacy Theseus export).
    if let Some(from_profile) = read_meta_from_profile_json(folder) {
        merge_meta(&mut meta, from_profile);
    }
    if let Some(from_mi) = read_meta_from_minecraftinstance(folder) {
        merge_meta(&mut meta, from_mi);
    }
    if let Some(from_manifest) = read_meta_from_manifest(folder) {
        merge_meta(&mut meta, from_manifest);
    }

    // 3) Heuristic loader only — never invent a Minecraft version.
    let (heur_loader, heur_gv, heur_lv) = detect_loader_heuristic(folder);
    if meta.loader.is_none() {
        meta.loader = Some(heur_loader);
    }
    if meta.game_version.is_none() {
        meta.game_version = heur_gv;
    }
    if meta.loader_version.is_none() {
        meta.loader_version = heur_lv;
    }

    meta
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
                base_path,
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

    import_modrinth_folder(
        folder,
        instance_folder,
        instance_id,
        base_path,
        reporter,
        details,
    )
    .await
}

async fn import_modrinth_folder(
    folder: PathBuf,
    folder_name: String,
    instance_id: &str,
    search_root: PathBuf,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let detected = resolve_meta(&folder, &folder_name, &search_root).await;

    let game_version = detected.game_version.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Could not detect Minecraft version for Modrinth profile `{folder_name}`. \
             Open Modrinth App once so app.db is up to date, or set the version manually after import."
        ))
    })?;

    let mod_loader = detected.loader.unwrap_or(ModLoader::Vanilla);
    let loader_version_hint = detected.loader_version;

    let name = detected
        .name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| folder_name.clone());

    let icon = if let Some(rel) = detected.icon {
        let p = if Path::new(&rel).is_absolute() {
            PathBuf::from(&rel)
        } else {
            folder.join(&rel)
        };
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
    finish_import(instance_id, folder, &state.io_semaphore, reporter, details)
        .await?;

    Ok(())
}
