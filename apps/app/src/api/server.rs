use crate::api::Result;
use theseus::prelude::*;
use theseus::server::{RunningServerInfo, ServerInstance, ServerSoftware};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("server")
        .invoke_handler(tauri::generate_handler![
            server_list,
            server_get,
            server_create,
            server_remove,
            server_install,
            server_start,
            server_stop,
            server_kill,
            server_send_command,
            server_get_log,
            server_get_running,
            server_is_running,
            server_get_config,
            server_set_config,
            server_get_versions,
        ])
        .build()
}

#[tauri::command]
pub async fn server_list() -> Result<Vec<ServerInstance>> {
    Ok(server::list().await?)
}

#[tauri::command]
pub async fn server_get(id: &str) -> Result<Option<ServerInstance>> {
    Ok(server::get(id).await?)
}

#[tauri::command]
pub async fn server_create(
    name: &str,
    software: ServerSoftware,
    minecraft_version: &str,
) -> Result<ServerInstance> {
    Ok(server::create(name, software, minecraft_version).await?)
}

#[tauri::command]
pub async fn server_remove(id: &str) -> Result<()> {
    Ok(server::remove(id).await?)
}

#[tauri::command]
pub async fn server_install(id: &str) -> Result<ServerInstance> {
    Ok(server::install(id).await?)
}

#[tauri::command]
pub async fn server_start(id: &str) -> Result<RunningServerInfo> {
    Ok(server::start(id).await?)
}

#[tauri::command]
pub async fn server_stop(id: &str) -> Result<()> {
    Ok(server::stop(id).await?)
}

#[tauri::command]
pub async fn server_kill(id: &str) -> Result<()> {
    Ok(server::kill(id).await?)
}

#[tauri::command]
pub async fn server_send_command(id: &str, command: &str) -> Result<()> {
    Ok(server::send_command(id, command).await?)
}

#[tauri::command]
pub async fn server_get_log(id: &str) -> Result<Vec<String>> {
    Ok(server::get_log(id).await?)
}

#[tauri::command]
pub async fn server_get_running() -> Result<Vec<RunningServerInfo>> {
    Ok(server::get_running().await?)
}

#[tauri::command]
pub async fn server_is_running(id: &str) -> Result<bool> {
    Ok(server::get_running()
        .await?
        .iter()
        .any(|s| s.id == id))
}

#[tauri::command]
pub async fn server_get_config(id: &str, file: &str) -> Result<String> {
    Ok(server::get_config(id, file).await?)
}

#[tauri::command]
pub async fn server_set_config(
    id: &str,
    file: &str,
    contents: &str,
) -> Result<()> {
    Ok(server::set_config(id, file, contents).await?)
}

#[tauri::command]
pub async fn server_get_versions(
    software: ServerSoftware,
) -> Result<Vec<String>> {
    Ok(server::get_versions(software).await?)
}
