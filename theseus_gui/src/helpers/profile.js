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

// Add empty default instance
export async function create() {
  return await invoke('profile_create')
}

// Remove a profile
export async function remove(path) {
  return await invoke('profile_remove', path)
}

// Get a profile by path
export async function get(path) {
  return await invoke('profile_get', path)
}

// Get a copy of the profile set
export async function list() {
  return await invoke('profile_list')
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path, credentials) {
  return await invoke('profile_run', path, credentials)
}

// Run Minecraft using a pathed profile
// Waits for end
export async function run_wait(path, credentials) {
  return await invoke('run_wait', path, credentials)
}

// Tries to kill a running minecraft process (if PID is still stored)
export async function kill(child_pid) {
  return await invoke('profile_kill', child_pid)
}

// Wait for a running minecraft process (a Child)
export async function wait_for(child_pid) {
  return await invoke('profile_wait_for', child_pid)
}
