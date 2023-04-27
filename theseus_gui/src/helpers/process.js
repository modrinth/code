/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/// Gets if a process has finished by UUID
/// Returns bool
export async function has_finished_by_uuid(uuid) {
  return await invoke('process_has_finished_by_uuid', { uuid })
}

/// Gets process exit status by UUID
/// Returns u32
export async function get_exit_status_by_uuid(uuid) {
  return await invoke('process_get_exit_status_by_uuid', { uuid })
}

/// Gets all process IDs
/// Returns [u32]
export async function get_all_uuids() {
  return await invoke('process_get_all_uuids')
}

/// Gets all running process IDs
/// Returns [u32]
export async function get_all_running_uuids() {
  return await invoke('process_get_all_running_uuids')
}

/// Gets all running process IDs with a given profile path
/// Returns [u32]
export async function get_uuids_by_profile_path(profilePath) {
  return await invoke('process_get_uuids_by_profile_path', { profilePath })
}

/// Gets all running process IDs with a given profile path
/// Returns [u32]
export async function get_all_running_profile_paths(profile_path) {
  return await invoke('process_get_all_running_profile_paths', { profile_path })
}

/// Gets all running process IDs with a given profile path
/// Returns [u32]
export async function get_all_running_profiles() {
  return await invoke('process_get_all_running_profiles')
}

/// Gets process stderr by UUID
/// Returns String
export async function get_stderr_by_uuid(uuid) {
  return await invoke('process_get_stderr_by_uuid', { uuid })
}

/// Gets process stdout by UUID
/// Returns String
export async function get_stdout_by_uuid(uuid) {
  return await invoke('process_get_stdout_by_uuid', { uuid })
}

/// Kills a process by UUID
export async function kill_by_uuid(uuid) {
  return await invoke('process_kill_by_uuid', { uuid })
}
