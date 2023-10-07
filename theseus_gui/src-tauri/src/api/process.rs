use crate::api::Result;
use theseus::prelude::*;
use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("process")
        .invoke_handler(tauri::generate_handler![
            process_has_finished_by_uuid,
            process_get_exit_status_by_uuid,
            process_get_all_uuids,
            process_get_all_running_uuids,
            process_get_uuids_by_profile_path,
            process_get_all_running_profile_paths,
            process_get_all_running_profiles,
            process_kill_by_uuid,
            process_wait_for_by_uuid,
        ])
        .build()
}

// Checks if a process has finished by process UUID
#[tauri::command]
pub async fn process_has_finished_by_uuid(uuid: Uuid) -> Result<bool> {
    Ok(process::has_finished_by_uuid(&uuid).await?)
}

// Gets process exit status by process UUID
#[tauri::command]
pub async fn process_get_exit_status_by_uuid(
    uuid: Uuid,
) -> Result<Option<i32>> {
    Ok(process::get_exit_status_by_uuid(&uuid).await?)
}

// Gets all process UUIDs
#[tauri::command]
pub async fn process_get_all_uuids() -> Result<Vec<Uuid>> {
    Ok(process::get_all_uuids().await?)
}

// Gets all running process UUIDs
#[tauri::command]
pub async fn process_get_all_running_uuids() -> Result<Vec<Uuid>> {
    Ok(process::get_all_running_uuids().await?)
}

// Gets all process UUIDs by profile path
#[tauri::command]
pub async fn process_get_uuids_by_profile_path(
    profile_path: ProfilePathId,
) -> Result<Vec<Uuid>> {
    Ok(process::get_uuids_by_profile_path(profile_path).await?)
}

// Gets the Profile paths of each *running* stored process in the state
#[tauri::command]
pub async fn process_get_all_running_profile_paths(
) -> Result<Vec<ProfilePathId>> {
    Ok(process::get_all_running_profile_paths().await?)
}

// Gets the Profiles (cloned) of each *running* stored process in the state
#[tauri::command]
pub async fn process_get_all_running_profiles() -> Result<Vec<Profile>> {
    Ok(process::get_all_running_profiles().await?)
}

// Kill a process by process UUID
#[tauri::command]
pub async fn process_kill_by_uuid(uuid: Uuid) -> Result<()> {
    Ok(process::kill_by_uuid(&uuid).await?)
}

// Wait for a process to finish by process UUID
#[tauri::command]
pub async fn process_wait_for_by_uuid(uuid: Uuid) -> Result<()> {
    Ok(process::wait_for_by_uuid(&uuid).await?)
}
