/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem, ContentOwner } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

import { install_to_existing_instance } from '@/helpers/pack'

import type {
	CacheBehaviour,
	ContentFile,
	ContentFileProjectType,
	GameInstance,
	InstanceLink,
	InstanceLoader,
} from './types'

// Add instance
/*
    name: String,           // the name of the instance
    game_version: String,   // the game version of the instance
    modloader: ModLoader,   // the modloader to use
    - ModLoader is an enum, with the following variants: Vanilla, Forge, Fabric, Quilt
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Path,  // the icon for the instance
*/

export async function create(
	name: string,
	gameVersion: string,
	modloader: InstanceLoader,
	loaderVersion: string | null,
	icon: string | null,
	skipInstall: boolean,
	link?: InstanceLink | null,
): Promise<string> {
	// Trim string name to avoid "Unable to find directory"
	name = name.trim()
	return await invoke('plugin:instance|instance_create', {
		name,
		gameVersion,
		modloader,
		loaderVersion,
		icon,
		skipInstall,
		link,
	})
}

export async function duplicate(instanceId: string): Promise<string> {
	return await invoke('plugin:instance|instance_duplicate', { instanceId })
}

export async function remove(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_remove', { instanceId })
}

export async function get(instanceId: string): Promise<GameInstance | null> {
	return await invoke('plugin:instance|instance_get', { instanceId })
}

export async function get_many(instanceIds: string[]): Promise<GameInstance[]> {
	return await invoke('plugin:instance|instance_get_many', { instanceIds })
}

export async function get_projects(
	instanceId: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<Record<string, ContentFile>> {
	return await invoke('plugin:instance|instance_get_projects', { instanceId, cacheBehaviour })
}

export async function get_installed_project_ids(instanceId: string): Promise<string[]> {
	return await invoke('plugin:instance|instance_get_installed_project_ids', { instanceId })
}

// Get content items with rich metadata for an instance
// Returns content items filtered to exclude modpack files (if linked),
// sorted alphabetically by project name
export async function get_content_items(
	instanceId: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<ContentItem[]> {
	return await invoke('plugin:instance|instance_get_content_items', { instanceId, cacheBehaviour })
}

// Linked modpack info returned from backend
export interface LinkedModpackInfo {
	project: Labrinth.Projects.v2.Project
	version: Labrinth.Versions.v2.Version
	owner: ContentOwner | null
	has_update: boolean
	update_version_id: string | null
	update_version: Labrinth.Versions.v2.Version | null
}

// Get linked modpack info for an instance
// Returns project, version, and owner information for the linked modpack,
// or null if the instance is not linked to a modpack
export async function get_linked_modpack_info(
	instanceId: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<LinkedModpackInfo | null> {
	return await invoke('plugin:instance|instance_get_linked_modpack_info', {
		instanceId,
		cacheBehaviour,
	})
}

// Get content items that are part of the linked modpack
// Returns the modpack's dependencies as ContentItem list
// Returns empty array if the instance is not linked to a modpack
export async function get_linked_modpack_content(
	instanceId: string,
	cacheBehaviour?: CacheBehaviour,
): Promise<ContentItem[]> {
	return await invoke('plugin:instance|instance_get_linked_modpack_content', {
		instanceId,
		cacheBehaviour,
	})
}

// Convert a list of dependencies into ContentItems with rich metadata
export async function get_dependencies_as_content_items(
	dependencies: Labrinth.Versions.v3.Dependency[],
	cacheBehaviour?: CacheBehaviour,
): Promise<ContentItem[]> {
	return await invoke('plugin:instance|instance_get_dependencies_as_content_items', {
		dependencies,
		cacheBehaviour,
	})
}

export async function get_full_path(instanceId: string): Promise<string> {
	return await invoke('plugin:instance|instance_get_full_path', { instanceId })
}

export async function get_mod_full_path(instanceId: string, projectPath: string): Promise<string> {
	return await invoke('plugin:instance|instance_get_mod_full_path', { instanceId, projectPath })
}

export interface JavaVersion {
	parsed_version: number
	version: string
	architecture: string
	path: string
}

export async function get_optimal_jre_key(instanceId: string): Promise<JavaVersion | null> {
	return await invoke('plugin:instance|instance_get_optimal_jre_key', { instanceId })
}

export async function list(): Promise<GameInstance[]> {
	return await invoke('plugin:instance|instance_list')
}

export async function check_installed(instanceId: string, projectId: string): Promise<boolean> {
	return await invoke('plugin:instance|instance_check_installed', { instanceId, projectId })
}

export async function check_installed_batch(projectId: string): Promise<Record<string, boolean>> {
	return await invoke('plugin:instance|instance_check_installed_batch', { projectId })
}

export async function install(instanceId: string, force: boolean): Promise<void> {
	return await invoke('plugin:instance|instance_install', { instanceId, force })
}

export async function update_all(instanceId: string): Promise<Record<string, string>> {
	return await invoke('plugin:instance|instance_update_all', { instanceId })
}

// Updates a specified project
export async function update_project(instanceId: string, projectPath: string): Promise<string> {
	return await invoke('plugin:instance|instance_update_project', { instanceId, projectPath })
}

// Add a project to an instance from a version
// Returns a path to the new project file
export type DownloadReason = 'standalone' | 'dependency' | 'modpack' | 'update'

export async function add_project_from_version(
	instanceId: string,
	versionId: string,
	reason: DownloadReason,
	dependentOnVersionId?: string,
): Promise<string> {
	return await invoke('plugin:instance|instance_add_project_from_version', {
		instanceId,
		versionId,
		reason,
		dependentOnVersionId,
	})
}

// Add a project to an instance from a path + project_type
// Returns a path to the new project file
export async function add_project_from_path(
	instanceId: string,
	projectPath: string,
	projectType?: ContentFileProjectType,
): Promise<string> {
	return await invoke('plugin:instance|instance_add_project_from_path', {
		instanceId,
		projectPath,
		projectType,
	})
}

// Toggle disabling a project
export async function toggle_disable_project(
	instanceId: string,
	projectPath: string,
): Promise<string> {
	return await invoke('plugin:instance|instance_toggle_disable_project', {
		instanceId,
		projectPath,
	})
}

// Remove a project
export async function remove_project(instanceId: string, projectPath: string): Promise<void> {
	return await invoke('plugin:instance|instance_remove_project', { instanceId, projectPath })
}

// Update a managed Modrinth instance to a specific version
export async function update_managed_modrinth_version(
	instanceId: string,
	versionId: string,
): Promise<void> {
	return await invoke('plugin:instance|instance_update_managed_modrinth_version', {
		instanceId,
		versionId,
	})
}

// Repair a managed Modrinth instance
export async function update_repair_modrinth(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_repair_managed_modrinth', { instanceId })
}

// Export an instance to .mrpack
// included_overrides is an array of paths to override folders to include (ie: 'mods', 'resource_packs')
// Version id is optional (ie: 1.1.5)
export async function export_instance_mrpack(
	instanceId: string,
	exportLocation: string,
	includedOverrides: string[],
	versionId?: string,
	description?: string,
	name?: string,
): Promise<void> {
	return await invoke('plugin:instance|instance_export_mrpack', {
		instanceId,
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
// allows selection for 'included_overrides' in export_instance_mrpack
export async function get_pack_export_candidates(instanceId: string): Promise<string[]> {
	return await invoke('plugin:instance|instance_get_pack_export_candidates', { instanceId })
}

// Run Minecraft using an instance
// Returns PID of child
export async function run(
	instanceId: string,
	serverAddress: string | null = null,
): Promise<unknown> {
	return await invoke('plugin:instance|instance_run', { instanceId, serverAddress })
}

export async function kill(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_kill', { instanceId })
}

// Edits an instance
export async function edit(instanceId: string, editInstance: Partial<GameInstance>): Promise<void> {
	return await invoke('plugin:instance|instance_edit', { instanceId, editInstance })
}

// Edits an instance's icon
export async function edit_icon(instanceId: string, iconPath: string | null): Promise<void> {
	return await invoke('plugin:instance|instance_edit_icon', { instanceId, iconPath })
}

export async function finish_install(instance: GameInstance): Promise<void> {
	if (instance.install_stage !== 'pack_installed') {
		if (instance.link) {
			await install_to_existing_instance(
				instance.link.project_id ?? instance.link.server_project_id ?? '',
				instance.link.version_id ?? instance.link.content_version_id ?? '',
				instance.name,
				instance.id,
			)
		}
	} else {
		await install(instance.id, false)
	}
}
