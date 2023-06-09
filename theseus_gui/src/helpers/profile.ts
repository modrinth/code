/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'
import { JavaVersion } from './jre'
import { LoaderVersion } from './manifest'
import { Hooks, JavaSettings, MemorySettings, WindowSize } from './settings'
import { ModLoader } from './utils'

export interface Profile {
  /** The name of the profile, and relative path to create */
  name: string
  /** The game version of the profile */
  game_version: string
  /** The modloader to use */
  modloader: ModLoader
  /** The modloader version to use, set to "latest", "stable", or the ID of your chosen loader */
  loader_version: LoaderVersion | string | 'latest' | 'stable'

  /**
    The icon for the profile.
    Icon is a path to an image file, which will be copied into the profile directory.
  */
  icon: string
}

export interface EditProfile {
  metadata?: EditProfileMetadata
  java?: JavaSettings
  memory?: MemorySettings
  resolution?: WindowSize
  hooks?: Hooks
}

export interface EditProfileMetadata {
  name?: string
  game_version?: string
  loader?: ModLoader
  loader_version?: LoaderVersion
  groups?: string[]
}

/** Add empty default instance */
export async function create_empty(): Promise<string> {
  return await invoke('profile_create_empty')
}

/** Add instance */
export async function create(
  name: string,
  gameVersion: string,
  modloader: string,
  loaderVersion?: string,
  icon?: string
): Promise<string> {
  return await invoke('profile_create', { name, gameVersion, modloader, loaderVersion, icon })
}

// Remove a profile
export async function remove(path: string) {
  return await invoke('profile_remove', { path })
}

// Get a profile by path
// Returns a Profile
export async function get(path: string, clearProjects?: boolean): Promise<Profile | undefined> {
  return await invoke('profile_get', { path, clearProjects })
}

// Get optimal java version from profile
// Returns a java version
export async function get_optimal_jre_key(path: string): Promise<JavaVersion | undefined> {
  return await invoke('profile_get_optimal_jre_key', { path })
}

// Get a copy of the profile set
// Returns hashmap of path -> Profile
export async function list(clearProjects?: boolean): Promise<Map<string, Profile>> {
  return await invoke('profile_list', { clearProjects })
}

export async function check_installed(path: string, projectId: string): Promise<boolean> {
  return await invoke('profile_check_installed', { path, projectId })
}

// Installs/Repairs a profile
export async function install(path: string) {
  return await invoke('profile_install', { path })
}

// Updates all of a profile's projects
export async function update_all(path: string) {
  return await invoke('profile_update_all', { path })
}

// Updates a specified project
export async function update_project(path: string, projectPath: string) {
  return await invoke('profile_update_project', { path, projectPath })
}

// Add a project to a profile from a version
// Returns a path to the new project file
export async function add_project_from_version(path: string, versionId: string) {
  return await invoke('profile_add_project_from_version', { path, versionId })
}

/**
  Add a project to a profile from a path + project_type
  @return A path to the new project file
*/
export async function add_project_from_path(
  path: string,
  projectPath: string,
  projectType: string
) {
  return await invoke('profile_add_project_from_path', { path, projectPath, projectType })
}

// Toggle disabling a project
export async function toggle_disable_project(path: string, projectPath: string): Promise<string> {
  return await invoke('profile_toggle_disable_project', { path, projectPath })
}

// Remove a project
export async function remove_project(path: string, projectPath: string) {
  return await invoke('profile_remove_project', { path, projectPath })
}

// Run Minecraft using a pathed profile
export async function run(path: string): Promise<number> {
  return await invoke('profile_run', { path })
}

// Run Minecraft using a pathed profile
export async function run_wait(path: string) {
  return await invoke('profile_run_wait', { path })
}

// Edits a profile
export async function edit(path: string, editProfile: EditProfile) {
  return await invoke('profile_edit', { path, editProfile })
}

// Edits a profile's icon
export async function edit_icon(path: string, iconPath?: string) {
  return await invoke('profile_edit_icon', { path, iconPath })
}
