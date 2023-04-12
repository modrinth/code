use std::path::{Path, PathBuf};

use crate::api::Result;
use theseus::{prelude::*, window_scoped};

// Checks if a process has finished by process PID
#[tauri::command]
pub async fn process_has_finished_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<bool> {
    Ok(window_scoped!(window, process::has_finished_by_pid(pid)).await?)
}

// Gets process exit status by process PID
#[tauri::command]
pub async fn process_get_exit_status_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<Option<i32>> {
    Ok(window_scoped!(window, process::get_exit_status_by_pid(pid)).await?)
}

// Gets all process PIDs
#[tauri::command]
pub async fn process_get_all_pids(window: tauri::Window) -> Result<Vec<u32>> {
    Ok(window_scoped!(window, process::get_all_pids()).await?)
}

// Gets all running process PIDs
#[tauri::command]
pub async fn process_get_all_running_pids(
    window: tauri::Window,
) -> Result<Vec<u32>> {
    Ok(window_scoped!(window, process::get_all_running_pids()).await?)
}

// Gets all process PIDs by profile path
#[tauri::command]
pub async fn process_get_pids_by_profile_path(
    window: tauri::Window,
    profile_path: &Path,
) -> Result<Vec<u32>> {
    Ok(
        window_scoped!(window, process::get_pids_by_profile_path(profile_path))
            .await?,
    )
}

// Gets the Profile paths of each *running* stored process in the state
#[tauri::command]
pub async fn process_get_all_running_profile_paths(
    window: tauri::Window,
) -> Result<Vec<PathBuf>> {
    Ok(
        window_scoped!(window, process::get_all_running_profile_paths())
            .await?,
    )
}

// Gets the Profiles (cloned) of each *running* stored process in the state
#[tauri::command]
pub async fn process_get_all_running_profiles(
    window: tauri::Window,
) -> Result<Vec<Profile>> {
    Ok(window_scoped!(window, process::get_all_running_profiles()).await?)
}

// Gets process stderr by process PID
#[tauri::command]
pub async fn process_get_stderr_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<String> {
    Ok(window_scoped!(window, process::get_stderr_by_pid(pid)).await?)
}

// Gets process stdout by process PID
#[tauri::command]
pub async fn process_get_stdout_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<String> {
    Ok(window_scoped!(window, process::get_stdout_by_pid(pid)).await?)
}

// Kill a process by process PID
#[tauri::command]
pub async fn process_kill_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<()> {
    Ok(window_scoped!(window, process::kill_by_pid(pid)).await?)
}

// Wait for a process to finish by process PID
#[tauri::command]
pub async fn process_wait_for_by_pid(
    window: tauri::Window,
    pid: u32,
) -> Result<()> {
    Ok(window_scoped!(window, process::wait_for_by_pid(pid)).await?)
}
