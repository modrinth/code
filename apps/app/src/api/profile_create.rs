use crate::api::Result;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile-create")
        .invoke_handler(tauri::generate_handler![
            profile_create,
            profile_duplicate
        ])
        .build()
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// invoke('plugin:profile-create|profile_add',profile)
#[tauri::command]
pub async fn profile_create(
    name: String,         // the name of the profile, and relative path
    game_version: String, // the game version of the profile
    modloader: ModLoader, // the modloader to use
    loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Option<String>,           // the icon for the profile
    skip_install: Option<bool>,
) -> Result<String> {
    let res = profile::create::profile_create(
        name,
        game_version,
        modloader,
        loader_version,
        icon,
        None,
        skip_install,
    )
    .await?;
    Ok(res)
}

// Creates a profile from a duplicate
// invoke('plugin:profile-create|profile_duplicate',profile)
#[tauri::command]
pub async fn profile_duplicate(path: &str) -> Result<String> {
    let res = profile::create::profile_create_from_duplicate(path).await?;
    Ok(res)
}
