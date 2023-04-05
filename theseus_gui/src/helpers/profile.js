/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Add empty default instance
export async function addDefaultInstance() {
  return await invoke('profile_create_empty')
}

/// Add empty default instance
/// Returns a path to the profile created
export async function create() {
  return await invoke('profile_create')
}

// Remove a profile
export async function remove(path) {
  return await invoke('profile_remove', { path })
}

// Get a profile by path
// Returns a Profile
export async function get(path) {
  return await invoke('profile_get', { path })
}

// Get a copy of the profile set
// Returns hashmap of path -> Profile
export async function list() {
  return await invoke('profile_list')
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path, credentials) {
  return await invoke('profile_run', { path, credentials })
}

// Run Minecraft using a pathed profile
// Waits for end
export async function run_wait(path, credentials) {
  return await invoke('profile_run_wait', { path, credentials })
}
