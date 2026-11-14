use std::path::PathBuf;

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::{
    ErrorKind, State,
    install::{InstallPhaseDetails, InstallProgressReporter},
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
    util::{fetch::fetch, io},
};

use super::{finish_import, recache_icon};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftInstance {
    pub name: Option<String>,
    pub base_mod_loader: Option<MinecraftInstanceModLoader>,
    pub profile_image_path: Option<PathBuf>,
    pub installed_modpack: Option<InstalledModpack>,
    pub game_version: String, // Minecraft game version. Non-prioritized, use this if Vanilla
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftInstanceModLoader {
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModpack {
    pub thumbnail_url: Option<String>,
}

// Check if folder has a minecraftinstance.json that parses
pub async fn is_valid_curseforge(instance_folder: PathBuf) -> bool {
    let minecraft_instance = serde_json::from_str::<MinecraftInstance>(
        &io::read_any_encoding_to_string(
            &instance_folder.join("minecraftinstance.json"),
        )
        .await
        .unwrap_or(("".into(), encoding_rs::UTF_8))
        .0,
    );
    minecraft_instance.is_ok()
}

fn parse_mod_loader<'a>(
    name: &'a str,
    game_version: &str,
) -> crate::Result<(ModLoader, &'a str)> {
    let (loader, version) = name.split_once('-').unwrap_or((name, ""));
    let mod_loader = match loader {
        "forge" => ModLoader::Forge,
        "neoforge" => ModLoader::NeoForge,
        "fabric" => ModLoader::Fabric,
        "quilt" => ModLoader::Quilt,
        _ => bail!(ErrorKind::InputError(format!(
            "unsupported CurseForge mod loader {name}"
        ))),
    };
    let version = version
        .strip_prefix(game_version)
        .and_then(|version| version.strip_prefix('-'))
        .unwrap_or(version);
    let version = version
        .strip_suffix(game_version)
        .and_then(|version| version.strip_suffix('-'))
        .unwrap_or(version);
    if version.is_empty() {
        bail!(ErrorKind::InputError(format!(
            "CurseForge mod loader {name} has no version"
        )));
    }
    Ok((mod_loader, version))
}

pub async fn import_curseforge(
    curseforge_instance_folder: PathBuf, // instance's folder
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    // Load minecraftinstance.json
    let minecraft_instance = serde_json::from_str::<MinecraftInstance>(
        &io::read_any_encoding_to_string(
            &curseforge_instance_folder.join("minecraftinstance.json"),
        )
        .await
        .unwrap_or(("".into(), encoding_rs::UTF_8))
        .0,
    )?;
    let override_title = minecraft_instance.name;
    let backup_name = format!(
        "Curseforge-{}",
        curseforge_instance_folder
            .file_name()
            .map_or("Unknown".to_string(), |a| a.to_string_lossy().to_string())
    );

    let state = State::get().await?;
    // Recache Curseforge Icon if it exists
    let mut icon = None;

    if let Some(icon_path) = minecraft_instance.profile_image_path.clone() {
        icon = recache_icon(icon_path).await?;
    } else if let Some(InstalledModpack {
        thumbnail_url: Some(thumbnail_url),
    }) = minecraft_instance.installed_modpack.clone()
    {
        let icon_bytes = fetch(
            &thumbnail_url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            &state.pool,
        )
        .await?;
        icon =
            Some(crate::api::instance::cache_icon(icon_bytes, &state).await?);
    }

    // base mod loader is always None for vanilla
    if let Some(instance_mod_loader) = minecraft_instance.base_mod_loader {
        let game_version = minecraft_instance.game_version;

        let (mod_loader, loader_version) =
            parse_mod_loader(&instance_mod_loader.name, &game_version)?;

        let loader_version = crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            Some(loader_version),
            &state.directories,
        )
        .await?;

        crate::api::instance::edit(
            instance_id,
            EditInstance {
                install_stage: Some(InstanceInstallStage::PackInstalling),
                name: Some(
                    override_title
                        .clone()
                        .unwrap_or_else(|| backup_name.to_string()),
                ),
                icon_path: Some(
                    icon.clone().map(|x| x.to_string_lossy().to_string()),
                ),
                content_set_patch: Some(AppliedContentSetPatch {
                    source_kind: None,
                    game_version: Some(game_version.clone()),
                    protocol_version: Some(None),
                    loader: Some(mod_loader),
                    loader_version: Some(loader_version.clone().map(|x| x.id)),
                }),
                ..EditInstance::default()
            },
        )
        .await?;
    } else {
        crate::api::instance::edit(
            instance_id,
            EditInstance {
                name: Some(
                    override_title
                        .clone()
                        .unwrap_or_else(|| backup_name.to_string()),
                ),
                icon_path: Some(
                    icon.clone().map(|x| x.to_string_lossy().to_string()),
                ),
                content_set_patch: Some(AppliedContentSetPatch {
                    source_kind: None,
                    game_version: Some(minecraft_instance.game_version.clone()),
                    protocol_version: Some(None),
                    loader: Some(ModLoader::Vanilla),
                    loader_version: Some(None),
                }),
                ..EditInstance::default()
            },
        )
        .await?;
    }

    // Copy in contained folders as overrides
    finish_import(
        instance_id,
        curseforge_instance_folder,
        &state,
        reporter,
        details,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_curseforge_mod_loader_names() {
        let parse = |name, game_version| {
            parse_mod_loader(name, game_version)
                .map(|(loader, version)| (loader, version.to_string()))
                .ok()
        };
        assert_eq!(
            parse("forge-47.2.0", "1.20.1"),
            Some((ModLoader::Forge, "47.2.0".to_string()))
        );
        assert_eq!(
            parse("fabric-0.16.9-1.21.1", "1.21.1"),
            Some((ModLoader::Fabric, "0.16.9".to_string()))
        );
        assert_eq!(
            parse("quilt-0.26.0-1.20.1", "1.20.1"),
            Some((ModLoader::Quilt, "0.26.0".to_string()))
        );
        assert_eq!(
            parse("neoforge-21.1.72", "1.21.1"),
            Some((ModLoader::NeoForge, "21.1.72".to_string()))
        );
        assert_eq!(
            parse("neoforge-1.20.1-47.1.106", "1.20.1"),
            Some((ModLoader::NeoForge, "47.1.106".to_string()))
        );
        assert_eq!(parse("liteloader-1.12.2", "1.12.2"), None);
        assert_eq!(parse("forge", "1.20.1"), None);
    }
}
