use crate::api::Result;
use std::path::PathBuf;
use theseus::prelude::*;

// Generic basic profile creation tool.
// Creates an essentially empty dummy profile with profile_create
#[tauri::command]
pub async fn profile_create_empty() -> Result<PathBuf> {
    let res = profile_create::profile_create_empty().await?;
    State::sync().await?;
    Ok(res)
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_create(
    name: String,           // the name of the profile, and relative path
    game_version: String,   // the game version of the profile
    modloader: ModLoader,   // the modloader to use
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Option<PathBuf>,  // the icon for the profile
) -> Result<PathBuf> {
    let res = profile_create::profile_create(
        name,
        game_version,
        modloader,
        loader_version,
        icon,
    )
    .await?;
    State::sync().await?;
    Ok(res)
}
