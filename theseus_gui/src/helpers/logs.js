/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/*
A log is a struct containing the filename string, stdout, and stderr, as follows:

pub struct Logs {
    pub filename:  String,
    pub stdout: String,
    pub stderr: String,
}
*/

/// Get all logs that exist for a given profile
/// This is returned as an array of Log objects, sorted by filename (the folder name, when the log was created)
export async function get_logs(profilePath, clearContents) {
  return await invoke('plugin:logs|logs_get_logs', { profilePath, clearContents })
}

/// Get a profile's log by filename
export async function get_logs_by_filename(profilePath, filename) {
  return await invoke('plugin:logs|logs_get_logs_by_filename', { profilePath, filename })
}

/// Get a profile's log text only by filename
export async function get_output_by_filename(profilePath, filename) {
  return await invoke('plugin:logs|logs_get_output_by_filename', { profilePath, filename })
}

/// Delete a profile's log by filename
export async function delete_logs_by_filename(profilePath, filename) {
  return await invoke('plugin:logs|logs_delete_logs_by_filename', { profilePath, filename })
}

/// Delete all logs for a given profile
export async function delete_logs(profilePath) {
  return await invoke('plugin:logs|logs_delete_logs', { profilePath })
}

/// Get the latest log for a given profile and cursor (startpoint to read withi nthe file)
/// Returns:
/*
  {
    cursor: u64
    output: String
    new_file: bool <- the cursor was too far, meaning that the file was likely rotated/reset. This signals to the frontend to clear the log and start over with this struct.
  }
*/
// From latest.log directly
export async function get_latest_log_cursor(profilePath, cursor) {
  return await invoke('plugin:logs|logs_get_latest_log_cursor', { profilePath, cursor })
}
// For std log (from modrinth app written latest_stdout.log, contains stdout and stderr)
export async function get_std_log_cursor(profilePath, cursor) {
  return await invoke('plugin:logs|logs_get_std_log_cursor', { profilePath, cursor })
}
