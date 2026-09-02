/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

/*
A log is a struct containing the filename string and optional output, as follows:

pub struct Logs {
    pub filename:  String,
    pub output: Option<String>,
}
*/

/// Get all logs that exist for a given instance
/// This is returned as an array of Log objects, sorted by filename (the folder name, when the log was created)
export async function get_logs(instanceId, clearContents) {
	return await invoke('plugin:logs|logs_get_logs', { instanceId, clearContents })
}

/// Get an instance's log by filename
export async function get_logs_by_filename(instanceId, logType, filename) {
	return await invoke('plugin:logs|logs_get_logs_by_filename', { instanceId, logType, filename })
}

/// Get an instance's log text only by filename
export async function get_output_by_filename(instanceId, logType, filename) {
	return await invoke('plugin:logs|logs_get_output_by_filename', {
		instanceId,
		logType,
		filename,
	})
}

/// Delete an instance's log by filename
export async function delete_logs_by_filename(instanceId, logType, filename) {
	return await invoke('plugin:logs|logs_delete_logs_by_filename', {
		instanceId,
		logType,
		filename,
	})
}

/// Delete all logs for a given instance
export async function delete_logs(instanceId) {
	return await invoke('plugin:logs|logs_delete_logs', { instanceId })
}

/// Get the latest log for a given instance and cursor (startpoint to read within the file)
/// Returns:
/*
  {
    cursor: u64
    output: String
    new_file: bool <- the cursor was too far, meaning that the file was likely rotated/reset. This signals to the frontend to clear the log and start over with this struct.
  }
*/

// From latest.log directly
export async function get_latest_log_cursor(instanceId, cursor) {
	return await invoke('plugin:logs|logs_get_latest_log_cursor', { instanceId, cursor })
}

/// Get all buffered live log lines for an instance from the Rust ring buffer
export async function get_live_log_buffer(instanceId) {
	return await invoke('plugin:logs|logs_get_live_log_buffer', { instanceId })
}

/// Clear the live log buffer for an instance on the Rust side
export async function clear_log_buffer(instanceId) {
	return await invoke('plugin:logs|logs_clear_live_log_buffer', { instanceId })
}
