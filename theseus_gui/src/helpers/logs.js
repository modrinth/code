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
export async function get_logs(profileUuid) {
  return await invoke('logs_get_logs', { profileUuid })
}

/// Get a profile's log by datetime_string (the folder name, when the log was created)
export async function get_log_by_datetime(profileUuid, datetimeString) {
  return await invoke('logs_get_log_by_datetime', { profileUuid, datetimeString })
}

/// Get a profile's stdout only by datetime_string (the folder name, when the log was created)
export async function get_stdout_by_datetime(profileUuid, datetimeString) {
  return await invoke('logs_get_stdout_by_datetime', { profileUuid, datetimeString })
}

/// Get a profile's stderr only by datetime_string (the folder name, when the log was created)
export async function get_stderr_by_datetime(profileUuid, datetimeString) {
  return await invoke('logs_get_stderr_by_datetime', { profileUuid, datetimeString })
}
