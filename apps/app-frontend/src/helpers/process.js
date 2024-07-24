/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/// Gets all running process IDs with a given profile path
/// Returns [u32]
export async function get_by_profile_path(path) {
  return await invoke('plugin:process|process_get_by_profile_path', { path })
}

/// Gets all running process IDs with a given profile path
/// Returns [u32]
export async function get_all() {
  return await invoke('plugin:process|process_get_all')
}

/// Kills a process by UUID
export async function kill(pid) {
  return await invoke('plugin:process|process_kill', { pid })
}
