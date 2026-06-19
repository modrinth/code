use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    State,
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
    util::io,
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
    let config = serde_json::from_str::<GDLauncherConfig>(
        &io::read_any_encoding_to_string(&instance_folder.join("config.json"))
            .await
            .unwrap_or(("".into(), encoding_rs::UTF_8))
            .0,
    );
    config.is_ok()
}

pub async fn import_gdlauncher(
    gdlauncher_instance_folder: PathBuf, // instance's folder
    instance_id: &str,
) -> crate::Result<()> {
    // Load config.json
    let config = serde_json::from_str::<GDLauncherConfig>(
        &io::read_any_encoding_to_string(
            &gdlauncher_instance_folder.join("config.json"),
        )
        .await
        .unwrap_or(("".into(), encoding_rs::UTF_8))
        .0,
    )?;
    let override_title = config.loader.source_name;
    let backup_name = format!(
        "GDLauncher-{}",
        gdlauncher_instance_folder
            .file_name()
            .map_or("Unknown".to_string(), |a| a.to_string_lossy().to_string())
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
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version.as_deref(),
        )
        .await?
    } else {
        None
    };

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
                game_version: Some(game_version.clone()),
                protocol_version: Some(None),
                loader: Some(mod_loader),
                loader_version: Some(loader_version.clone().map(|x| x.id)),
            }),
            ..EditInstance::default()
        },
    )
    .await?;

    // Copy in contained folders as overrides
    let state = State::get().await?;
    let loading_bar = copy_dotminecraft(
        instance_id,
        gdlauncher_instance_folder,
        &state.io_semaphore,
        None,
    )
    .await?;

    crate::launcher::install_minecraft_for_instance_id(
        instance_id,
        Some(loading_bar),
        false,
    )
    .await?;

    Ok(())
}
