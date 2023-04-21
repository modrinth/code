/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Add empty default instance
export async function create_empty() {
  return await invoke('profile_create_empty')
}

/// Add instance
/*
    name: String,           // the name of the profile, and relative path to create
    game_version: String,   // the game version of the profile
    modloader: ModLoader,   // the modloader to use
    - ModLoader is an enum, with the following variants: Vanilla, Forge, Fabric, Quilt
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Path,  // the icon for the profile
    - icon is a path to an image file, which will be copied into the profile directory
*/

export async function create(name, game_version, modloader, loader_version, icon) {
  return await invoke('profile_create', { name, game_version, modloader, loader_version, icon })
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

// Add a project to a profile from a version
// Returns a path to the new project file
export async function addProjectFromVersion(path, version_id) {
  return await invoke('profile_add_project_from_version', { path, version_id })
}

// Add a project to a profile from a path + project_type
// Returns a path to the new project file
export async function add_project_from_path(path, project_path, project_type) {
  return await invoke('profile_add_project_from_path', { path, project_path, project_type })
}

// Toggle disabling a project
export async function toggle_disable_project(path, projectPath) {
  return await invoke('profile_toggle_disable_project', { path, projectPath })
}

// Remove a project
export async function remove_project(path, projectPath) {
  return await invoke('profile_remove_project', { path, projectPath })
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path) {
  return await invoke('profile_run', { path })
}

// Run Minecraft using a pathed profile
// Waits for end
export async function run_wait(path) {
  return await invoke('profile_run_wait', { path })
}
