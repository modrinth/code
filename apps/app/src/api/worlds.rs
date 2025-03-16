use crate::api::Result;
use either::Either;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager, Runtime};
use theseus::prelude::ProcessMetadata;
use theseus::profile::QuickPlayType;
use theseus::worlds::{ServerStatus, World};
use theseus::{profile, worlds, State};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("worlds")
        .invoke_handler(tauri::generate_handler![
            get_profile_worlds,
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
    let state = State::get().await?;
    let path = state.directories.profiles_dir().join(path);
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
pub async fn get_server_status(address: &str) -> Result<ServerStatus> {
    Ok(worlds::get_server_status(address).await?)
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
    let (host, port) = worlds::parse_server_address(address)?;
    let process =
        profile::run(path, &QuickPlayType::Server(host.to_owned(), port))
            .await?;

    Ok(process)
}
