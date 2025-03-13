use crate::api::Result;
use theseus::worlds;
use theseus::worlds::{ServerStatus, World};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("worlds")
        .invoke_handler(tauri::generate_handler![
            get_profile_worlds,
            get_server_status,
        ])
        .build()
}

#[tauri::command]
pub async fn get_profile_worlds(path: &str) -> Result<Vec<World>> {
    Ok(worlds::get_profile_worlds(path).await?)
}

#[tauri::command]
pub async fn get_server_status(address: &str) -> Result<ServerStatus> {
    Ok(worlds::get_server_status(address).await?)
}
