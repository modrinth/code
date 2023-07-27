/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/*
A log is a struct containing the datetime string, stdout, and stderr, as follows:

pub struct Logs {
    pub datetime_string:  String,
    pub stdout: String,
    pub stderr: String,
}
*/

/// Get all logs that exist for a given profile
/// This is returned as an array of Log objects, sorted by datetime_string (the folder name, when the log was created)
export async function get_logs(profilePath, clearContents) {
  return await invoke('plugin:logs|logs_get_logs', { profilePath, clearContents })
}

/// Get a profile's log by datetime_string (the folder name, when the log was created)
export async function get_logs_by_datetime(profilePath, datetimeString) {
  return await invoke('plugin:logs|logs_get_logs_by_datetime', { profilePath, datetimeString })
}

/// Get a profile's stdout only by datetime_string (the folder name, when the log was created)
export async function get_output_by_datetime(profilePath, datetimeString) {
  return await invoke('plugin:logs|logs_get_output_by_datetime', { profilePath, datetimeString })
}

/// Delete a profile's log by datetime_string (the folder name, when the log was created)
export async function delete_logs_by_datetime(profilePath, datetimeString) {
  return await invoke('plugin:logs|logs_delete_logs_by_datetime', { profilePath, datetimeString })
}

/// Delete all logs for a given profile
export async function delete_logs(profilePath) {
  return await invoke('plugin:logs|logs_delete_logs', { profilePath })
}
