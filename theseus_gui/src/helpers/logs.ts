/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/** A log is a struct containing the datetime string, stdout, and stderr, as follows: */
export interface Logs {
  datetime_string: string
  stdout: string
  stderr: string
}

/**
  Get all logs that exist for a given profile
  @return An array of Log objects, sorted by datetime_string (the folder name, when the log was created)
*/
export async function get_logs(profileUuid: string, clearContents?: boolean): Promise<Logs[]> {
  return await invoke('logs_get_logs', { profileUuid, clearContents })
}

/** Get a profile's log by datetime_string (the folder name, when the log was created) */
export async function get_logs_by_datetime(
  profileUuid: string,
  datetimeString: string
): Promise<Logs> {
  return await invoke('logs_get_logs_by_datetime', { profileUuid, datetimeString })
}

/** Get a profile's stdout only by datetime_string (the folder name, when the log was created) */
export async function get_stdout_by_datetime(
  profileUuid: string,
  datetimeString: string
): Promise<string> {
  return await invoke('logs_get_stdout_by_datetime', { profileUuid, datetimeString })
}

/** Get a profile's stderr only by datetime_string (the folder name, when the log was created) */
export async function get_stderr_by_datetime(
  profileUuid: string,
  datetimeString: string
): Promise<string> {
  return await invoke('logs_get_stderr_by_datetime', { profileUuid, datetimeString })
}

/** Delete a profile's log by datetime_string (the folder name, when the log was created) */
export async function delete_logs_by_datetime(profileUuid: string, datetimeString: string) {
  return await invoke('logs_delete_logs_by_datetime', { profileUuid, datetimeString })
}

/** Delete all logs for a given profile */
export async function delete_logs(profileUuid: string) {
  return await invoke('logs_delete_logs', { profileUuid })
}
