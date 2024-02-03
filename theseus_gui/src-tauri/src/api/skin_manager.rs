use std::{collections::HashMap, path::PathBuf};

use crate::api::Result;
use theseus::{skin_manager::{self, SkinSave, SkinCache}, auth::Credentials};

use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("skin")
        .invoke_handler(tauri::generate_handler![
            skin_get_user_skin_data,
            skin_get_heads,
            skin_set_skin,
            skin_delete_skin,
            skin_import_skin,
            skin_check_image,
            skin_get_cape_data,
            skin_cache_users_skins,
            skin_cache_new_user_skin,
            skin_get_mojang_launcher_path,
            skin_get_mojang_launcher_names,
            skin_save_skin,
            skin_get_skins,
            skin_set_cape
        ])
        .build()
}

// Get image size
// invoke('plugin:skin|check_image',path)
#[tauri::command]
pub async fn skin_check_image(path: String) -> Result<bool> {
    Ok(skin_manager::check_image(path).await?)
}

// Sets player's skin
// invoke('plugin:skin|skin_set_skin', { skin, arms, user })
#[tauri::command]
pub async fn skin_set_skin(skin: String, arms: String, user: Credentials) -> Result<bool> {
    Ok(skin_manager::set_skin(skin, arms, user).await?)
}

// Sets the players cape
// invoke('plugin:skin|skin_set_cape', { capeid, token })
#[tauri::command]
pub async fn skin_set_cape(capeid: String, token: String) -> Result<bool> {
    Ok(skin_manager::set_cape(capeid, token).await?)
}

// Returns cape info
// invoke('plugin:skin|skin_set_cape', { capeid, token })
#[tauri::command]
pub async fn skin_get_cape_data(cape: String, key: String) -> Result<String> {
    Ok(skin_manager::get_cape_data(cape, key).await?)
}

// Gets the current skin, cape, and updates the unlocked cape list
// invoke('plugin:skin|skin_get_player_info', { user, token })
#[tauri::command]
pub async fn skin_get_user_skin_data(id: Uuid) -> Result<SkinCache> {
    Ok(skin_manager::get_user_skin_data(id).await?)
}

// 
// 
#[tauri::command]
pub async fn skin_cache_users_skins() -> Result<bool> {
    Ok(skin_manager::cache_users_skins().await?)
}

// 
// 
#[tauri::command]
pub async fn skin_cache_new_user_skin(user: Credentials) -> Result<bool> {
    Ok(skin_manager::cache_new_user_skin(user).await?)
}

#[tauri::command]
pub async fn skin_save_skin(user: Uuid, data: SkinCache, name: String, model: String, skinid: String) -> Result<bool> {
    Ok(skin_manager::save_skin(user, data, name, model, skinid).await?)
}

#[tauri::command]
pub async fn skin_delete_skin(id: Uuid) -> Result<bool> {
    Ok(skin_manager::delete_skin(id).await?)
}

#[tauri::command]
pub async fn skin_get_heads() -> Result<HashMap<Uuid, String>> {
    Ok(skin_manager::get_heads().await?)
}

#[tauri::command]
pub async fn skin_get_skins() -> Result<Vec<SkinSave>> {
    Ok(skin_manager::get_skins().await?)
}

#[tauri::command]
pub async fn skin_get_mojang_launcher_path() -> Result<PathBuf> {
    Ok(skin_manager::get_mojang_launcher_path().await?)
}

#[tauri::command]
pub async fn skin_get_mojang_launcher_names(path: PathBuf) -> Result<Vec<skin_manager::MojangNames>> {
    Ok(skin_manager::get_mojang_launcher_names(path).await?)
}

#[tauri::command]
pub async fn skin_import_skin(name: String, path: PathBuf, user: Uuid) -> Result<()> {
    Ok(skin_manager::import_skin(name, path, user).await?)
}