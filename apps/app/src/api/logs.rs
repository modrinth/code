use crate::api::Result;
use theseus::logs::LogType;
use theseus::logs::{self, CensoredString, LatestLogCursor, Logs};

/*
A log is a struct containing the filename string, stdout, and stderr, as follows:

pub struct Logs {
    pub filename:  String,
    pub stdout: String,
    pub stderr: String,
}
*/

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("logs")
        .invoke_handler(tauri::generate_handler![
            logs_get_logs,
            logs_get_logs_by_filename,
            logs_get_output_by_filename,
            logs_delete_logs,
            logs_delete_logs_by_filename,
            logs_get_latest_log_cursor,
            logs_get_live_log_buffer,
            logs_clear_live_log_buffer,
        ])
        .build()
}

/// Get all logs for an instance, sorted by filename.
#[tauri::command]
pub async fn logs_get_logs(
    instance_id: &str,
    clear_contents: Option<bool>,
) -> Result<Vec<Logs>> {
    let val = logs::get_logs(instance_id, clear_contents).await?;

    Ok(val)
}

/// Get a log struct for an instance by filename.
#[tauri::command]
pub async fn logs_get_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: String,
) -> Result<Logs> {
    Ok(logs::get_logs_by_filename(instance_id, log_type, filename).await?)
}

/// Get the output for an instance by filename.
#[tauri::command]
pub async fn logs_get_output_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: String,
) -> Result<CensoredString> {
    Ok(logs::get_output_by_filename(instance_id, log_type, &filename).await?)
}

/// Delete all logs for an instance.
#[tauri::command]
pub async fn logs_delete_logs(instance_id: &str) -> Result<()> {
    Ok(logs::delete_logs(instance_id).await?)
}

/// Delete a log for an instance by filename.
#[tauri::command]
pub async fn logs_delete_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: String,
) -> Result<()> {
    Ok(logs::delete_logs_by_filename(instance_id, log_type, &filename).await?)
}

/// Get live log from a cursor
#[tauri::command]
pub async fn logs_get_latest_log_cursor(
    instance_id: &str,
    cursor: u64, // 0 to start at beginning of file
) -> Result<LatestLogCursor> {
    Ok(logs::get_latest_log_cursor(instance_id, cursor).await?)
}

/// Get all buffered live log lines for an instance.
#[tauri::command]
pub async fn logs_get_live_log_buffer(
    instance_id: &str,
) -> Result<CensoredString> {
    Ok(logs::get_live_log_buffer(instance_id).await?)
}

/// Clear the live log buffer for an instance.
#[tauri::command]
pub async fn logs_clear_live_log_buffer(instance_id: &str) -> Result<()> {
    logs::clear_live_log_buffer(instance_id);
    Ok(())
}
