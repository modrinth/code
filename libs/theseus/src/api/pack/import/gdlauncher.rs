use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    prelude::{ModLoader, Profile, ProfilePathId},
    state::ProfileInstallStage,
    util::io,
    State,
};

use super::{copy_dotminecraft, recache_icon};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GDLauncherConfig {
    pub background: Option<String>,
    pub loader: GDLauncherLoader,
    // pub mods: Vec<GDLauncherMod>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GDLauncherLoader {
    pub loader_type: ModLoader,
    pub loader_version: Option<String>,
    pub mc_version: String,
    pub source: Option<String>,
    pub source_name: Option<String>,
}

// Check if folder has a config.json that parses
pub async fn is_valid_gdlauncher(instance_folder: PathBuf) -> bool {
    let config: String =
        io::read_to_string(&instance_folder.join("config.json"))
            .await
            .unwrap_or("".to_string());
    let config: Result<GDLauncherConfig, serde_json::Error> =
        serde_json::from_str::<GDLauncherConfig>(&config);
    config.is_ok()
}

pub async fn import_gdlauncher(
    gdlauncher_instance_folder: PathBuf, // instance's folder
    profile_path: ProfilePathId,         // path to profile
) -> crate::Result<()> {
    // Load config.json
    let config: String =
        io::read_to_string(&gdlauncher_instance_folder.join("config.json"))
            .await?;
    let config: GDLauncherConfig =
        serde_json::from_str::<GDLauncherConfig>(&config)?;
    let override_title: Option<String> = config.loader.source_name.clone();
    let backup_name = format!(
        "GDLauncher-{}",
        gdlauncher_instance_folder
            .file_name()
            .map(|a| a.to_string_lossy().to_string())
            .unwrap_or("Unknown".to_string())
    );

    // Re-cache icon
    let icon = config
        .background
        .clone()
        .map(|b| gdlauncher_instance_folder.join(b));
    let icon = if let Some(icon) = icon {
        recache_icon(icon).await?
    } else {
        None
    };

    let game_version = config.loader.mc_version;
    let mod_loader = config.loader.loader_type;
    let loader_version = config.loader.loader_version;

    let loader_version = if mod_loader != ModLoader::Vanilla {
        crate::profile::create::get_loader_version_from_loader(
            game_version.clone(),
            mod_loader,
            loader_version,
        )
        .await?
    } else {
        None
    };

    // Set profile data to created default profile
    crate::api::profile::edit(&profile_path, |prof| {
        prof.metadata.name = override_title
            .clone()
            .unwrap_or_else(|| backup_name.to_string());
        prof.install_stage = ProfileInstallStage::PackInstalling;
        prof.metadata.icon = icon.clone();
        prof.metadata.game_version = game_version.clone();
        prof.metadata.loader_version = loader_version.clone();
        prof.metadata.loader = mod_loader;

        async { Ok(()) }
    })
    .await?;

    // Copy in contained folders as overrides
    let state = State::get().await?;
    let loading_bar = copy_dotminecraft(
        profile_path.clone(),
        gdlauncher_instance_folder,
        &state.io_semaphore,
        None,
    )
    .await?;

    if let Some(profile_val) =
        crate::api::profile::get(&profile_path, None).await?
    {
        crate::launcher::install_minecraft(
            &profile_val,
            Some(loading_bar),
            false,
        )
        .await?;
        {
            let state = State::get().await?;
            let mut file_watcher = state.file_watcher.write().await;
            Profile::watch_fs(
                &profile_val.get_profile_full_path().await?,
                &mut file_watcher,
            )
            .await?;
        }
        State::sync().await?;
    }

    Ok(())
}
