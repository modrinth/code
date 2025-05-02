use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    State,
    prelude::ModLoader,
    state::ProfileInstallStage,
    util::{
        fetch::{fetch, write_cached_icon},
        io,
    },
};

use super::{copy_dotminecraft, recache_icon};

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
    let minecraftinstance: String =
        io::read_to_string(&instance_folder.join("minecraftinstance.json"))
            .await
            .unwrap_or("".to_string());
    let minecraftinstance: Result<MinecraftInstance, serde_json::Error> =
        serde_json::from_str::<MinecraftInstance>(&minecraftinstance);
    minecraftinstance.is_ok()
}

pub async fn import_curseforge(
    curseforge_instance_folder: PathBuf, // instance's folder
    profile_path: &str,                  // path to profile
) -> crate::Result<()> {
    // Load minecraftinstance.json
    let minecraft_instance: String = io::read_to_string(
        &curseforge_instance_folder.join("minecraftinstance.json"),
    )
    .await?;
    let minecraft_instance: MinecraftInstance =
        serde_json::from_str::<MinecraftInstance>(&minecraft_instance)?;
    let override_title: Option<String> = minecraft_instance.name.clone();
    let backup_name = format!(
        "Curseforge-{}",
        curseforge_instance_folder
            .file_name()
            .map(|a| a.to_string_lossy().to_string())
            .unwrap_or("Unknown".to_string())
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
        let icon_bytes =
            fetch(&thumbnail_url, None, &state.fetch_semaphore, &state.pool)
                .await?;
        let filename = thumbnail_url.rsplit('/').next_back();
        if let Some(filename) = filename {
            icon = Some(
                write_cached_icon(
                    filename,
                    &state.directories.caches_dir(),
                    icon_bytes,
                    &state.io_semaphore,
                )
                .await?,
            );
        }
    }

    // base mod loader is always None for vanilla
    if let Some(instance_mod_loader) = minecraft_instance.base_mod_loader {
        let game_version = minecraft_instance.game_version;

        // CF allows Forge, Fabric, and Vanilla
        let mut mod_loader = None;
        let mut loader_version = None;

        match instance_mod_loader.name.split('-').collect::<Vec<&str>>()[..] {
            ["forge", version] => {
                mod_loader = Some(ModLoader::Forge);
                loader_version = Some(version.to_string());
            }
            ["fabric", version, _game_version] => {
                mod_loader = Some(ModLoader::Fabric);
                loader_version = Some(version.to_string());
            }
            _ => {}
        }

        let mod_loader = mod_loader.unwrap_or(ModLoader::Vanilla);

        let loader_version = if mod_loader != ModLoader::Vanilla {
            crate::launcher::get_loader_version_from_profile(
                &game_version,
                mod_loader,
                loader_version.as_deref(),
            )
            .await?
        } else {
            None
        };

        // Set profile data to created default profile
        crate::api::profile::edit(profile_path, |prof| {
            prof.name = override_title
                .clone()
                .unwrap_or_else(|| backup_name.to_string());
            prof.install_stage = ProfileInstallStage::PackInstalling;
            prof.icon_path =
                icon.clone().map(|x| x.to_string_lossy().to_string());
            prof.game_version.clone_from(&game_version);
            prof.loader_version = loader_version.clone().map(|x| x.id);
            prof.loader = mod_loader;

            async { Ok(()) }
        })
        .await?;
    } else {
        // create a vanilla profile
        crate::api::profile::edit(profile_path, |prof| {
            prof.name = override_title
                .clone()
                .unwrap_or_else(|| backup_name.to_string());
            prof.icon_path =
                icon.clone().map(|x| x.to_string_lossy().to_string());
            prof.game_version
                .clone_from(&minecraft_instance.game_version);
            prof.loader_version = None;
            prof.loader = ModLoader::Vanilla;

            async { Ok(()) }
        })
        .await?;
    }

    // Copy in contained folders as overrides
    let state = State::get().await?;
    let loading_bar = copy_dotminecraft(
        profile_path,
        curseforge_instance_folder,
        &state.io_semaphore,
        None,
    )
    .await?;

    if let Some(profile_val) = crate::api::profile::get(profile_path).await? {
        crate::launcher::install_minecraft(
            &profile_val,
            Some(loading_bar),
            false,
        )
        .await?;
    }

    Ok(())
}
