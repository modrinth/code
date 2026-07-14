/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem, ContentOwner } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

import type { InstallJobSnapshot } from './install'
import type {
	CacheBehaviour,
	ContentFile,
	ContentFileProjectType,
	GameInstance,
	InstanceLoader,
} from './types'

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

export type InstanceInstallTarget = {
	game_version: string
	loader: string
}

export type InstanceInstallCandidate = {
	id: string
	name: string
	icon_path?: string | null
	game_version: string
	loader: InstanceLoader
	installed: boolean
	compatible: boolean
}

export async function get_install_candidates(
	projectId: string,
	projectType: string,
	targets: InstanceInstallTarget[],
): Promise<InstanceInstallCandidate[]> {
	return await invoke('plugin:instance|instance_get_install_candidates', {
		projectId,
		projectType,
		targets,
	})
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

export interface ResolutionPreferences {
	game_versions?: string[]
	loaders?: string[]
}

export interface ResolveContentRequest {
	project_id: string
	version_id?: string | null
	content_type: Labrinth.Content.v3.ContentType
	selected?: ResolutionPreferences
}

export interface ResolvedContent {
	project_id: string
	version_id: string
	dependent_on_version_id?: string | null
}

export interface ResolveContentPlan {
	primary: ResolvedContent
	dependencies: ResolvedContent[]
	skipped: Array<{
		project_id: string
		version_id?: string | null
		dependent_on_version_id?: string | null
		reason: string
	}>
}

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

export async function install_project_with_dependencies(
	instanceId: string,
	request: ResolveContentRequest,
): Promise<ResolveContentPlan> {
	return await invoke('plugin:instance|instance_install_project_with_dependencies', {
		instanceId,
		request,
	})
}

export async function switch_project_version_with_dependencies(
	instanceId: string,
	projectPath: string,
	versionId: string,
): Promise<string> {
	return await invoke('plugin:instance|instance_switch_project_version_with_dependencies', {
		instanceId,
		projectPath,
		versionId,
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

export async function is_file_on_modrinth(projectPath: string): Promise<boolean> {
	return await invoke('plugin:instance|instance_is_file_on_modrinth', { projectPath })
}

// Toggle disabling a project
export async function toggle_disable_project(
	instanceId: string,
	projectPath: string,
	desiredEnabled?: boolean,
): Promise<string> {
	return await invoke('plugin:instance|instance_toggle_disable_project', {
		instanceId,
		projectPath,
		desiredEnabled,
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
): Promise<InstallJobSnapshot> {
	return await invoke('plugin:instance|instance_update_managed_modrinth_version', {
		instanceId,
		versionId,
	})
}

// Repair a managed Modrinth instance
export async function update_repair_modrinth(instanceId: string): Promise<InstallJobSnapshot> {
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
