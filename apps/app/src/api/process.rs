use crate::api::Result;
use theseus::prelude::*;

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
pub async fn process_get_all() -> Result<Vec<Process>> {
    Ok(process::get_all().await?)
}

#[tauri::command]
pub async fn process_get_by_profile_path(path: &str) -> Result<Vec<Process>> {
    Ok(process::get_by_profile_path(path).await?)
}

#[tauri::command]
pub async fn process_kill(pid: i32) -> Result<()> {
    Ok(process::kill(pid).await?)
}

#[tauri::command]
pub async fn process_wait_for(pid: i32) -> Result<()> {
    Ok(process::wait_for(pid).await?)
}
