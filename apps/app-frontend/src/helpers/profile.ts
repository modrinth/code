/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem, ContentOwner } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

import { install_to_existing_profile } from '@/helpers/pack.js'

import type {
	CacheBehaviour,
	ContentFile,
	ContentFileProjectType,
	GameInstance,
	InstanceLoader,
} from './types'

// Add instance
/*
    name: String,           // the name of the profile, and relative path to create
    game_version: String,   // the game version of the profile
    modloader: ModLoader,   // the modloader to use
    - ModLoader is an enum, with the following variants: Vanilla, Forge, Fabric, Quilt
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Path,  // the icon for the profile
    - icon is a path to an image file, which will be copied into the profile directory
*/

export async function create(
	name: string,
	gameVersion: string,
	modloader: InstanceLoader,
	loaderVersion: string | null,
	icon: string | null,
	skipInstall: boolean,
): Promise<string> {
	// Trim string name to avoid "Unable to find directory"
	name = name.trim()
	return await invoke('plugin:profile-create|profile_create', {
		name,
		gameVersion,
		modloader,
		loaderVersion,
		icon,
		skipInstall,
	})
}

// duplicate a profile
export async function duplicate(path: string): Promise<string> {
	return await invoke('plugin:profile-create|profile_duplicate', { path })
}

// Remove a profile
export async function remove(path: string): Promise<void> {
	return await invoke('plugin:profile|profile_remove', { path })
}

// Get a profile by path
// Returns a Profile
export async function get(path: string): Promise<GameInstance | null> {
	return await invoke('plugin:profile|profile_get', { path })
}

export async function get_many(paths: string[]): Promise<GameInstance[]> {
	return await invoke('plugin:profile|profile_get_many', { paths })
}

// Get a profile's projects
// Returns a map of a path to profile file
export async function get_projects(
	path: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<Record<string, ContentFile>> {
	return await invoke('plugin:profile|profile_get_projects', { path, cacheBehaviour })
}

// Get content items with rich metadata for a profile
// Returns content items filtered to exclude modpack files (if linked),
// sorted alphabetically by project name
export async function get_content_items(
	path: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<ContentItem[]> {
	return await invoke('plugin:profile|profile_get_content_items', { path, cacheBehaviour })
}

// Linked modpack info returned from backend
export interface LinkedModpackInfo {
	project: Labrinth.Projects.v2.Project
	version: Labrinth.Versions.v2.Version
	owner: ContentOwner | null
}

// Get linked modpack info for a profile
// Returns project, version, and owner information for the linked modpack,
// or null if the profile is not linked to a modpack
export async function get_linked_modpack_info(
	path: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<LinkedModpackInfo | null> {
	return await invoke('plugin:profile|profile_get_linked_modpack_info', { path, cacheBehaviour })
}

// Get a profile's full fs path
// Returns a path
export async function get_full_path(path: string): Promise<string> {
	return await invoke('plugin:profile|profile_get_full_path', { path })
}

// Get's a mod's full fs path
// Returns a path
export async function get_mod_full_path(path: string, projectPath: string): Promise<string> {
	return await invoke('plugin:profile|profile_get_mod_full_path', { path, projectPath })
}

// Get optimal java version from profile
// Returns a java version
export async function get_optimal_jre_key(path: string): Promise<string | null> {
	return await invoke('plugin:profile|profile_get_optimal_jre_key', { path })
}

// Get a copy of the profile set
// Returns hashmap of path -> Profile
export async function list(): Promise<GameInstance[]> {
	return await invoke('plugin:profile|profile_list')
}

export async function check_installed(path: string, projectId: string): Promise<boolean> {
	return await invoke('plugin:profile|profile_check_installed', { path, projectId })
}

// Installs/Repairs a profile
export async function install(path: string, force: boolean): Promise<void> {
	return await invoke('plugin:profile|profile_install', { path, force })
}

// Updates all of a profile's projects
export async function update_all(path: string): Promise<Record<string, string>> {
	return await invoke('plugin:profile|profile_update_all', { path })
}

// Updates a specified project
export async function update_project(path: string, projectPath: string): Promise<string> {
	return await invoke('plugin:profile|profile_update_project', { path, projectPath })
}

// Add a project to a profile from a version
// Returns a path to the new project file
export async function add_project_from_version(path: string, versionId: string): Promise<string> {
	return await invoke('plugin:profile|profile_add_project_from_version', { path, versionId })
}

// Add a project to a profile from a path + project_type
// Returns a path to the new project file
export async function add_project_from_path(
	path: string,
	projectPath: string,
	projectType?: ContentFileProjectType,
): Promise<string> {
	return await invoke('plugin:profile|profile_add_project_from_path', {
		path,
		projectPath,
		projectType,
	})
}

// Toggle disabling a project
export async function toggle_disable_project(path: string, projectPath: string): Promise<string> {
	return await invoke('plugin:profile|profile_toggle_disable_project', { path, projectPath })
}

// Remove a project
export async function remove_project(path: string, projectPath: string): Promise<void> {
	return await invoke('plugin:profile|profile_remove_project', { path, projectPath })
}

// Update a managed Modrinth profile to a specific version
export async function update_managed_modrinth_version(
	path: string,
	versionId: string,
): Promise<void> {
	return await invoke('plugin:profile|profile_update_managed_modrinth_version', {
		path,
		versionId,
	})
}

// Repair a managed Modrinth profile
export async function update_repair_modrinth(path: string): Promise<void> {
	return await invoke('plugin:profile|profile_repair_managed_modrinth', { path })
}

// Export a profile to .mrpack
// included_overrides is an array of paths to override folders to include (ie: 'mods', 'resource_packs')
// Version id is optional (ie: 1.1.5)
export async function export_profile_mrpack(
	path: string,
	exportLocation: string,
	includedOverrides: string[],
	versionId?: string,
	description?: string,
	name?: string,
): Promise<void> {
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
export async function get_pack_export_candidates(profilePath: string): Promise<string[]> {
	return await invoke('plugin:profile|profile_get_pack_export_candidates', { profilePath })
}

// Run Minecraft using a pathed profile
// Returns PID of child
export async function run(path: string): Promise<unknown> {
	return await invoke('plugin:profile|profile_run', { path })
}

export async function kill(path: string): Promise<void> {
	return await invoke('plugin:profile|profile_kill', { path })
}

// Edits a profile
export async function edit(path: string, editProfile: Partial<GameInstance>): Promise<void> {
	return await invoke('plugin:profile|profile_edit', { path, editProfile })
}

// Edits a profile's icon
export async function edit_icon(path: string, iconPath: string | null): Promise<void> {
	return await invoke('plugin:profile|profile_edit_icon', { path, iconPath })
}

export async function finish_install(instance: GameInstance): Promise<void> {
	if (instance.install_stage !== 'pack_installed') {
		const linkedData = instance.linked_data
		if (linkedData) {
			await install_to_existing_profile(
				linkedData.project_id,
				linkedData.version_id,
				instance.name,
				instance.path,
			)
		}
	} else {
		await install(instance.path, false)
	}
}
