use crate::api::Result;
use either::Either;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager, Runtime};
use theseus::prelude::ProcessMetadata;
use theseus::profile::{get_full_path, QuickPlayType};
use theseus::worlds::{ServerPackStatus, ServerStatus, World};
use theseus::{profile, worlds, State};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("worlds")
        .invoke_handler(tauri::generate_handler![
            get_profile_worlds,
            rename_world,
            reset_world_icon,
            add_server_to_profile,
            get_profile_protocol_version,
            get_server_status,
            start_join_singleplayer_world,
            start_join_server,
        ])
        .build()
}

#[tauri::command]
pub async fn get_profile_worlds<R: Runtime>(
    app_handle: AppHandle<R>,
    path: &str,
) -> Result<Vec<World>> {
    let path = get_full_path(path).await?;
    let mut result = worlds::get_profile_worlds(&path).await?;
    for world in result.iter_mut() {
        if let Some(icon) = &world.icon {
            if let Either::Left(icon_path) = &icon {
                let icon_path = icon_path.clone();
                if let Ok(new_url) =
                    super::utils::tauri_convert_file_src(&icon_path)
                {
                    world.icon = Some(Either::Right(new_url));
                    if let Err(e) =
                        app_handle.asset_protocol_scope().allow_file(&icon_path)
                    {
                        tracing::warn!(
                            "Failed to allow file access for icon {}: {}",
                            icon_path.display(),
                            e
                        );
                    }
                } else {
                    tracing::warn!(
                        "Encountered invalid icon path for world {}: {}",
                        world.name,
                        icon_path.display()
                    );
                    world.icon = None;
                }
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn rename_world(
    instance: &str,
    world: &str,
    new_name: &str,
) -> Result<()> {
    let instance = get_full_path(instance).await?;
    worlds::rename_world(&instance, world, new_name).await?;
    Ok(())
}

#[tauri::command]
pub async fn reset_world_icon(instance: &str, world: &str) -> Result<()> {
    let instance = get_full_path(instance).await?;
    worlds::reset_world_icon(&instance, world).await?;
    Ok(())
}

#[tauri::command]
pub async fn add_server_to_profile(
    path: &str,
    name: String,
    address: String,
    pack_status: ServerPackStatus,
) -> Result<()> {
    let path = get_full_path(path).await?;
    Ok(
        worlds::add_server_to_profile(&path, name, address, pack_status)
            .await?,
    )
}

#[tauri::command]
pub async fn get_profile_protocol_version(path: &str) -> Result<Option<i32>> {
    Ok(worlds::get_profile_protocol_version(path).await?)
}

#[tauri::command]
pub async fn get_server_status(
    address: &str,
    protocol_version: Option<i32>,
) -> Result<ServerStatus> {
    Ok(worlds::get_server_status(address, protocol_version).await?)
}

#[tauri::command]
pub async fn start_join_singleplayer_world(
    path: &str,
    world: String,
) -> Result<ProcessMetadata> {
    let process =
        profile::run(path, &QuickPlayType::Singleplayer(world)).await?;

    Ok(process)
}

#[tauri::command]
pub async fn start_join_server(
    path: &str,
    address: &str,
) -> Result<ProcessMetadata> {
    let process =
        profile::run(path, &QuickPlayType::Server(address.to_owned())).await?;

    Ok(process)
}
