/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

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

export async function create(name, gameVersion, modloader, loaderVersion, icon, noWatch) {
  //Trim string name to avoid "Unable to find directory"
  name = name.trim()
  return await invoke('plugin:profile_create|profile_create', {
    name,
    gameVersion,
    modloader,
    loaderVersion,
    icon,
    noWatch,
  })
}

// duplicate a profile
export async function duplicate(path) {
  return await invoke('plugin:profile_create|profile_duplicate', { path })
}

// Remove a profile
export async function remove(path) {
  return await invoke('plugin:profile|profile_remove', { path })
}

// Get a profile by path
// Returns a Profile
export async function get(path, clearProjects) {
  return await invoke('plugin:profile|profile_get', { path, clearProjects })
}

// Get a profile's full fs path
// Returns a path
export async function get_full_path(path) {
  return await invoke('plugin:profile|profile_get_full_path', { path })
}

// Get's a mod's full fs path
// Returns a path
export async function get_mod_full_path(path, projectPath) {
  return await invoke('plugin:profile|profile_get_mod_full_path', { path, projectPath })
}

// Get optimal java version from profile
// Returns a java version
export async function get_optimal_jre_key(path) {
  return await invoke('plugin:profile|profile_get_optimal_jre_key', { path })
}

// Get a copy of the profile set
// Returns hashmap of path -> Profile
export async function list(clearProjects) {
  return await invoke('plugin:profile|profile_list', { clearProjects })
}

export async function check_installed(path, projectId) {
  return await invoke('plugin:profile|profile_check_installed', { path, projectId })
}

// Installs/Repairs a profile
export async function install(path, force) {
  return await invoke('plugin:profile|profile_install', { path, force })
}

// Updates all of a profile's projects
export async function update_all(path) {
  return await invoke('plugin:profile|profile_update_all', { path })
}

// Updates a specified project
export async function update_project(path, projectPath) {
  return await invoke('plugin:profile|profile_update_project', { path, projectPath })
}

// Add a project to a profile from a version
// Returns a path to the new project file
export async function add_project_from_version(path, versionId) {
  return await invoke('plugin:profile|profile_add_project_from_version', { path, versionId })
}

// Add a project to a profile from a path + project_type
// Returns a path to the new project file
export async function add_project_from_path(path, projectPath, projectType) {
  return await invoke('plugin:profile|profile_add_project_from_path', {
    path,
    projectPath,
    projectType,
  })
}

// Toggle disabling a project
export async function toggle_disable_project(path, projectPath) {
  return await invoke('plugin:profile|profile_toggle_disable_project', { path, projectPath })
}

// Remove a project
export async function remove_project(path, projectPath) {
  return await invoke('plugin:profile|profile_remove_project', { path, projectPath })
}

// Update a managed Modrinth profile to a specific version
export async function update_managed_modrinth_version(path, versionId) {
  return await invoke('plugin:profile|profile_update_managed_modrinth_version', { path, versionId })
}

// Repair a managed Modrinth profile
export async function update_repair_modrinth(path) {
  return await invoke('plugin:profile|profile_repair_managed_modrinth', { path })
}

// Export a profile to .mrpack
/// included_overrides is an array of paths to override folders to include (ie: 'mods', 'resource_packs')
// Version id is optional (ie: 1.1.5)
export async function export_profile_mrpack(
  path,
  exportLocation,
  includedOverrides,
  versionId,
  description,
  name,
) {
  return await invoke('plugin:profile|profile_export_mrpack', {
    path,
    exportLocation,
    includedOverrides,
    versionId,
    description,
    name,
  })
}

// Given a folder path, populate an array of all the subfolders
// Intended to be used for finding potential override folders
// profile
// -- mods
// -- resourcepacks
// -- file1
// => [mods, resourcepacks]
// allows selection for 'included_overrides' in export_profile_mrpack
export async function get_pack_export_candidates(profilePath) {
  return await invoke('plugin:profile|profile_get_pack_export_candidates', { profilePath })
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path) {
  return await invoke('plugin:profile|profile_run', { path })
}

// Run Minecraft using a pathed profile
// Waits for end
export async function run_wait(path) {
  return await invoke('plugin:profile|profile_run_wait', { path })
}

// Edits a profile
export async function edit(path, editProfile) {
  return await invoke('plugin:profile|profile_edit', { path, editProfile })
}

// Edits a profile's icon
export async function edit_icon(path, iconPath) {
  return await invoke('plugin:profile|profile_edit_icon', { path, iconPath })
}
