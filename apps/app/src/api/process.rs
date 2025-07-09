use crate::api::Result;
use theseus::prelude::*;
use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("process")
        .invoke_handler(tauri::generate_handler![
            process_get_all,
            process_get_by_profile_path,
            process_kill,
            process_wait_for,
        ])
        .build()
}

#[tauri::command]
pub async fn process_get_all() -> Result<Vec<ProcessMetadata>> {
    Ok(process::get_all().await?)
}

#[tauri::command]
pub async fn process_get_by_profile_path(
    path: &str,
) -> Result<Vec<ProcessMetadata>> {
    Ok(process::get_by_profile_path(path).await?)
}

#[tauri::command]
pub async fn process_kill(uuid: Uuid) -> Result<()> {
    Ok(process::kill(uuid).await?)
}

#[tauri::command]
pub async fn process_wait_for(uuid: Uuid) -> Result<()> {
    Ok(process::wait_for(uuid).await?)
}
