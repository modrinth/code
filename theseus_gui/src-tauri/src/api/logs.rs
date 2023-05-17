use crate::api::Result;
use theseus::logs::{self, Logs};
use uuid::Uuid;

/*
A log is a struct containing the datetime string, stdout, and stderr, as follows:

pub struct Logs {
    pub datetime_string:  String,
    pub stdout: String,
    pub stderr: String,
}
*/

/// Get all Logs for a profile, sorted by datetime
#[tauri::command]
pub async fn logs_get_logs(
    profile_uuid: Uuid,
    clear_contents: Option<bool>,
) -> Result<Vec<Logs>> {
    use std::time::Instant;
    let now = Instant::now();
    let val = logs::get_logs(profile_uuid, clear_contents).await?;
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(val)
}

/// Get a Log struct for a profile by profile id and datetime string
#[tauri::command]
pub async fn logs_get_logs_by_datetime(
    profile_uuid: Uuid,
    datetime_string: String,
) -> Result<Logs> {
    Ok(logs::get_logs_by_datetime(profile_uuid, datetime_string).await?)
}

/// Get the stdout for a profile by profile id and datetime string
#[tauri::command]
pub async fn logs_get_stdout_by_datetime(
    profile_uuid: Uuid,
    datetime_string: String,
) -> Result<String> {
    Ok(logs::get_stdout_by_datetime(profile_uuid, &datetime_string).await?)
}

/// Get the stderr for a profile by profile id and datetime string
#[tauri::command]
pub async fn logs_get_stderr_by_datetime(
    profile_uuid: Uuid,
    datetime_string: String,
) -> Result<String> {
    Ok(logs::get_stderr_by_datetime(profile_uuid, &datetime_string).await?)
}

/// Delete all logs for a profile by profile id
#[tauri::command]
pub async fn logs_delete_logs(profile_uuid: Uuid) -> Result<()> {
    Ok(logs::delete_logs(profile_uuid).await?)
}

/// Delete a log for a profile by profile id and datetime string
#[tauri::command]
pub async fn logs_delete_logs_by_datetime(
    profile_uuid: Uuid,
    datetime_string: String,
) -> Result<()> {
    Ok(logs::delete_logs_by_datetime(profile_uuid, &datetime_string).await?)
}
