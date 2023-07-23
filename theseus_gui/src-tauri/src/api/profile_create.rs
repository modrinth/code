use crate::api::Result;
use std::path::PathBuf;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile_create")
        .invoke_handler(tauri::generate_handler![profile_create,])
        .build()
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// invoke('plugin:profile|profile_add',profile)
#[tauri::command]
pub async fn profile_create(
    name: String,         // the name of the profile, and relative path
    game_version: String, // the game version of the profile
    modloader: ModLoader, // the modloader to use
    loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Option<PathBuf>,          // the icon for the profile
) -> Result<ProfilePathId> {
    let res = profile_create::profile_create(
        name,
        game_version,
        modloader,
        loader_version,
        icon,
        None,
        None,
        None,
    )
    .await?;
    Ok(res)
}
