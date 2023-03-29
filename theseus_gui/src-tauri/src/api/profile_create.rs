use crate::api::Result;
use std::path::PathBuf;
use theseus::prelude::*;

// Generic basic profile creation tool.
// Creates an essentially empty dummy profile with profile_create
#[tauri::command]
pub async fn profile_create_empty() -> Result<Profile> {
    let res = profile_create::profile_create_empty().await?;
    State::sync().await?;
    Ok(res)
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// This is reused mostly from the CLI. TODO: touch up.
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_create(
    path: PathBuf,          // the path of the newly created profile
    name: String,           // the name of the profile
    game_version: String,   // the game version of the profile
    icon: Option<PathBuf>,  // the icon for the profile
    modloader: ModLoader,   // the modloader to use
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
) -> Result<Profile> {
    let res = profile_create::profile_create(path, name, game_version, icon, modloader, loader_version).await?;
    State::sync().await?;
    Ok(res)
}
