use crate::api::Result;
use std::path::PathBuf;
use theseus::{prelude::*, profile_create::CreatePackProfile};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile_create")
        .invoke_handler(tauri::generate_handler![
            profile_get_profile_empty,
            profile_create,
        ])
        .build()
}

// Generic basic profile creation tool.
// Creates an essentially empty dummy profile for use with profile_create
#[tauri::command]
pub async fn profile_get_profile_empty() -> Result<CreatePackProfile> {
    let res = profile_create::get_profile_empty();
    Ok(res)
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
) -> Result<PathBuf> {
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
