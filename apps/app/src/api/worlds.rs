use crate::api::Result;
use either::Either;
use enumset::EnumSet;
use tauri::{AppHandle, Manager, Runtime};
use theseus::prelude::ProcessMetadata;
use theseus::profile::{QuickPlayType, get_full_path};
use theseus::worlds::{
    DisplayStatus, ServerPackStatus, ServerStatus, World, WorldType,
    WorldWithProfile,
};
use theseus::{profile, worlds};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("worlds")
        .invoke_handler(tauri::generate_handler![
            get_recent_worlds,
            get_profile_worlds,
            get_singleplayer_world,
            set_world_display_status,
            rename_world,
            reset_world_icon,
            backup_world,
            delete_world,
            add_server_to_profile,
            edit_server_in_profile,
            remove_server_from_profile,
            get_profile_protocol_version,
            get_server_status,
            start_join_singleplayer_world,
            start_join_server,
        ])
        .build()
}

#[tauri::command]
pub async fn get_recent_worlds<R: Runtime>(
    app_handle: AppHandle<R>,
    limit: usize,
    display_statuses: Option<EnumSet<DisplayStatus>>,
) -> Result<Vec<WorldWithProfile>> {
    let mut result = worlds::get_recent_worlds(
        limit,
        display_statuses.unwrap_or(EnumSet::all()),
    )
    .await?;
    for world in result.iter_mut() {
        adapt_world_icon(&app_handle, &mut world.world);
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_profile_worlds<R: Runtime>(
    app_handle: AppHandle<R>,
    path: &str,
) -> Result<Vec<World>> {
    let mut result = worlds::get_profile_worlds(path).await?;
    for world in result.iter_mut() {
        adapt_world_icon(&app_handle, world);
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_singleplayer_world<R: Runtime>(
    app_handle: AppHandle<R>,
    instance: &str,
    world: &str,
) -> Result<World> {
    let mut world = worlds::get_singleplayer_world(instance, world).await?;
    adapt_world_icon(&app_handle, &mut world);
    Ok(world)
}

fn adapt_world_icon<R: Runtime>(app_handle: &AppHandle<R>, world: &mut World) {
    if let Some(Either::Left(icon_path)) = &world.icon {
        let icon_path = icon_path.clone();
        if let Ok(new_url) = super::utils::tauri_convert_file_src(&icon_path) {
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

#[tauri::command]
pub async fn set_world_display_status(
    instance: &str,
    world_type: WorldType,
    world_id: &str,
    display_status: DisplayStatus,
) -> Result<()> {
    Ok(worlds::set_world_display_status(
        instance,
        world_type,
        world_id,
        display_status,
    )
    .await?)
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
pub async fn backup_world(instance: &str, world: &str) -> Result<u64> {
    let instance = get_full_path(instance).await?;
    Ok(worlds::backup_world(&instance, world).await?)
}

#[tauri::command]
pub async fn delete_world(instance: &str, world: &str) -> Result<()> {
    let instance = get_full_path(instance).await?;
    worlds::delete_world(&instance, world).await?;
    Ok(())
}

#[tauri::command]
pub async fn add_server_to_profile(
    path: &str,
    name: String,
    address: String,
    pack_status: ServerPackStatus,
) -> Result<usize> {
    let path = get_full_path(path).await?;
    Ok(
        worlds::add_server_to_profile(&path, name, address, pack_status)
            .await?,
    )
}

#[tauri::command]
pub async fn edit_server_in_profile(
    path: &str,
    index: usize,
    name: String,
    address: String,
    pack_status: ServerPackStatus,
) -> Result<()> {
    let path = get_full_path(path).await?;
    worlds::edit_server_in_profile(&path, index, name, address, pack_status)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_server_from_profile(
    path: &str,
    index: usize,
) -> Result<()> {
    let path = get_full_path(path).await?;
    worlds::remove_server_from_profile(&path, index).await?;
    Ok(())
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
