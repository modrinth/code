use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    prelude::{ModLoader, ProfilePathId},
    state::ProfileInstallStage,
    util::io,
    State,
};

use super::copy_dotminecraft;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameManifest {
    pub manifest_version: u8,
    pub name: String,
    pub minecraft: FlameMinecraft,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameMinecraft {
    pub version: String,
    pub mod_loaders: Vec<FlameModLoader>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftInstance {
    pub name: Option<String>,
    pub game_version: String, // Minecraft game version. Non-prioritized, use this if Vanilla
}

// Check if folder has a minecraftinstance.json that parses
pub async fn is_valid_curseforge(instance_folder: PathBuf) -> bool {
    let minecraftinstance: String =
        fs::read_to_string(&instance_folder.join("minecraftinstance.json"))
            .await
            .unwrap_or("".to_string());
    let minecraftinstance: Result<MinecraftInstance, serde_json::Error> =
        serde_json::from_str::<MinecraftInstance>(&minecraftinstance);
    minecraftinstance.is_ok()
}

pub async fn import_curseforge(
    curseforge_instance_folder: PathBuf, // instance's folder
    profile_path: ProfilePathId,         // path to profile
) -> crate::Result<()> {
    // TODO: recache curseforge instance icon
    let icon: Option<PathBuf> = None;

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

    // Curseforge vanilla profile may not have a manifest.json, so we allow it to not exist
    if curseforge_instance_folder.join("manifest.json").exists() {
        // Load manifest.json
        let cf_manifest: String = io::read_to_string(
            &curseforge_instance_folder.join("manifest.json"),
        )
        .await?;

        let cf_manifest: FlameManifest =
            serde_json::from_str::<FlameManifest>(&cf_manifest)?;

        let game_version = cf_manifest.minecraft.version;

        // CF allows Forge, Fabric, and Vanilla
        let mut mod_loader = None;
        let mut loader_version = None;
        for loader in cf_manifest.minecraft.mod_loaders {
            match loader.id.split_once('-') {
                Some(("forge", version)) => {
                    mod_loader = Some(ModLoader::Forge);
                    loader_version = Some(version.to_string());
                }
                Some(("fabric", version)) => {
                    mod_loader = Some(ModLoader::Fabric);
                    loader_version = Some(version.to_string());
                }
                _ => {}
            }
        }
        let mod_loader = mod_loader.unwrap_or(ModLoader::Vanilla);

        let loader_version = if mod_loader != ModLoader::Vanilla {
            crate::profile_create::get_loader_version_from_loader(
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
    } else {
        // If no manifest is found, it's a vanilla profile
        crate::api::profile::edit(&profile_path, |prof| {
            prof.metadata.name = override_title
                .clone()
                .unwrap_or_else(|| backup_name.to_string());
            prof.metadata.icon = icon.clone();
            prof.metadata.game_version =
                minecraft_instance.game_version.clone();
            prof.metadata.loader_version = None;
            prof.metadata.loader = ModLoader::Vanilla;

            async { Ok(()) }
        })
        .await?;
    }

    // Copy in contained folders as overrides
    copy_dotminecraft(profile_path.clone(), curseforge_instance_folder).await?;

    if let Some(profile_val) =
        crate::api::profile::get(&profile_path, None).await?
    {
        crate::launcher::install_minecraft(&profile_val, None).await?;

        State::sync().await?;
    }

    Ok(())
}
