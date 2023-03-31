/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri';

// Add empty default instance
export async function addDefaultInstance() {
    return await invoke('profile_create_empty');
}

// Add empty default instance
export async function create() {
    return await invoke('profile_create');
}

// Add a profile to the in-memory state
export async function add(profile) {
    return await invoke('profile_add', profile);
}

// Add a path as a profile in-memory
export async function add_path(path) {
    return await invoke('profile_add_path', path);
}

// Remove a profile
export async function remove(path) {
    return await invoke('profile_remove', path);
}

// Get a profile by path
export async function get(path) {
    return await invoke('profile_get', path);
}

// Check if a pathed profile is already managed by Theseus
export async function is_managed(path) {
    return await invoke('profile_is_managed', path);
}

// Check if a pathed profile is loaded
export async function is_loaded(path) {
    return await invoke('profile_is_loaded', path);
}

// Get a copy of the profile set
export async function list() {
    return await invoke('profile_list');
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path, credentials) {
    return await invoke('profile_run', path, credentials);
}

// Run Minecraft using a pathed profile
export async function run(path_wait, credentials) {
    return await invoke('path_wait', path, credentials);
}

// Tries to kill a running minecraft process (if PID is still stored)
export async function kill(child_pid) {
    return await invoke('profile_kill', child_pid);
}

// Wait for a running minecraft process (a Child)
export async function wait_for(child_pid) {
    return await invoke('profile_wait_for', child_pid);
}
