/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import type { Labrinth } from '@modrinth/api-client'
import type { ContentItem, ContentOwner } from '@modrinth/ui'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'

import type { InstallJobSnapshot, SharedInstanceUpdateDiff } from './install'
import type {
	CacheBehaviour,
	ContentFile,
	ContentFileProjectType,
	GameInstance,
	InstanceIconConfig,
	InstanceLoader,
	SharedInstanceAttachment,
} from './types'

export function getInstanceIconUrl(iconPath: string | null | undefined): string | null {
	if (!iconPath) return null
	if (iconPath.startsWith('http://') || iconPath.startsWith('https://')) return iconPath
	return convertFileSrc(iconPath)
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
	const items = await invoke<ContentItem[]>('plugin:instance|instance_get_content_items', {
		instanceId,
		cacheBehaviour,
	})
	return adaptContentItems(items)
}

export async function refresh_content_updates(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_refresh_content_updates', { instanceId })
}

// Linked modpack info returned from backend
export interface LinkedModpackInfo {
	project: Labrinth.Projects.v2.Project
	version: Labrinth.Versions.v2.Version | null
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
	const items = await invoke<ContentItem[]>('plugin:instance|instance_get_linked_modpack_content', {
		instanceId,
		cacheBehaviour,
	})
	return adaptContentItems(items)
}

// Convert a list of dependencies into ContentItems with rich metadata
export async function get_dependencies_as_content_items(
	dependencies: Labrinth.Versions.v3.Dependency[],
	cacheBehaviour?: CacheBehaviour,
): Promise<ContentItem[]> {
	const items = await invoke<ContentItem[]>(
		'plugin:instance|instance_get_dependencies_as_content_items',
		{
			dependencies,
			cacheBehaviour,
		},
	)
	return adaptContentItems(items)
}

function adaptContentItems(items: ContentItem[]): ContentItem[] {
	return items.map((item) => {
		const embeddedMetadata = item.embedded_metadata
		if (!embeddedMetadata?.icon_path) return item

		return {
			...item,
			embedded_metadata: {
				...embeddedMetadata,
				icon_url: convertFileSrc(embeddedMetadata.icon_path),
			},
		}
	})
}

export async function get_full_path(instanceId: string): Promise<string> {
	return await invoke('plugin:instance|instance_get_full_path', { instanceId })
}

export async function get_mod_full_path(instanceId: string, projectPath: string): Promise<string> {
	return await invoke('plugin:instance|instance_get_mod_full_path', { instanceId, projectPath })
}

export type ScreenshotKey = {
	instance_id: string
	file_name: string
}

export type InstanceScreenshot = ScreenshotKey & {
	id: string
	instance_name: string
	created_at: string
	modified_at: number
	group_id?: string | null
	path: string
	url: string
}

export type ScreenshotGroup = {
	id: string
	name: string
}

export type ScreenshotGroupMembershipUpdate = {
	screenshot_id: string
	group_id?: string | null
}

export type ScreenshotGroupImport = ScreenshotGroup & {
	screenshot_ids: string[]
}

export async function list_instance_screenshots(instanceId: string): Promise<InstanceScreenshot[]> {
	return await invoke('plugin:instance|instance_list_screenshots', { instanceId })
}

export async function list_all_screenshots(): Promise<InstanceScreenshot[]> {
	return await invoke('plugin:instance|instance_list_all_screenshots')
}

export async function list_synced_screenshots(): Promise<InstanceScreenshot[]> {
	return await invoke('plugin:instance|instance_list_synced_screenshots')
}

export async function save_edited_screenshot(
	key: ScreenshotKey,
	pngBytes: Uint8Array,
	mode: 'create_copy' | 'replace_edit',
): Promise<InstanceScreenshot> {
	return await invoke('plugin:instance|instance_save_edited_screenshot', {
		key,
		pngBytes: Array.from(pngBytes),
		mode,
	})
}

export async function list_screenshot_groups(): Promise<ScreenshotGroup[]> {
	return await invoke('plugin:instance|instance_list_screenshot_groups')
}

export async function create_screenshot_group(
	name: string,
	screenshotIds: string[],
): Promise<ScreenshotGroup> {
	return await invoke('plugin:instance|instance_create_screenshot_group', {
		name,
		screenshotIds,
	})
}

export async function rename_screenshot_group(
	id: string,
	newName: string,
): Promise<ScreenshotGroup> {
	return await invoke('plugin:instance|instance_rename_screenshot_group', { id, newName })
}

export async function delete_screenshot_group(id: string): Promise<void> {
	return await invoke('plugin:instance|instance_delete_screenshot_group', { id })
}

export async function set_screenshot_group_memberships(
	updates: ScreenshotGroupMembershipUpdate[],
): Promise<void> {
	return await invoke('plugin:instance|instance_set_screenshot_group_memberships', { updates })
}

export async function import_screenshot_groups(groups: ScreenshotGroupImport[]): Promise<void> {
	return await invoke('plugin:instance|instance_import_screenshot_groups', { groups })
}

export async function delete_screenshots(keys: ScreenshotKey[]): Promise<void> {
	return await invoke('plugin:instance|instance_delete_screenshots', { keys })
}

export async function export_screenshots(keys: ScreenshotKey[], exportPath: string): Promise<void> {
	return await invoke('plugin:instance|instance_export_screenshots', { keys, exportPath })
}

export async function move_screenshots(
	keys: ScreenshotKey[],
	targetInstanceId: string,
): Promise<ScreenshotKey[]> {
	return await invoke('plugin:instance|instance_move_screenshots', { keys, targetInstanceId })
}

export async function open_screenshot(key: ScreenshotKey): Promise<void> {
	return await invoke('plugin:instance|instance_open_screenshot', { key })
}

export async function set_synced_option(
	instanceId: string,
	option: SyncedOption,
	enabled: boolean,
	resolution?: SyncedOptionJoinResolution,
): Promise<GameInstance> {
	return await invoke('plugin:instance|instance_set_synced_option', {
		instanceId,
		option,
		enabled,
		resolution,
	})
}

export type SyncedOption =
	| 'command_history'
	| 'multiplayer_servers'
	| 'creative_hotbars'
	| 'screenshots'

export type GlobalSyncedOptions = Record<SyncedOption, boolean>

export type SyncedOptionJoinAction = 'seed_shared' | 'attach' | 'merge' | 'requires_resolution'

export type SyncedOptionJoinResolution = 'use_synced' | 'use_instance'

export type SyncedOptionJoinPreview = {
	action: SyncedOptionJoinAction
}

export type SyncedOptionCapability = {
	option: SyncedOption
	supported: boolean
	disabled_reason?: string | null
}

export type SyncedOptionsOverview = {
	global_options: GlobalSyncedOptions
	capabilities: SyncedOptionCapability[]
}

export async function get_synced_option_join_preview(
	instanceId: string,
	option: SyncedOption,
): Promise<SyncedOptionJoinPreview> {
	return await invoke('plugin:instance|instance_get_synced_option_join_preview', {
		instanceId,
		option,
	})
}

export async function get_synced_options_overview(
	instanceId: string,
): Promise<SyncedOptionsOverview> {
	return await invoke('plugin:instance|instance_get_synced_options_overview', { instanceId })
}

export async function get_global_synced_options(): Promise<GlobalSyncedOptions> {
	return await invoke('plugin:instance|instance_get_global_synced_options')
}

export async function set_global_synced_option(
	option: SyncedOption,
	enabled: boolean,
): Promise<GlobalSyncedOptions> {
	return await invoke('plugin:instance|instance_set_global_synced_option', {
		option,
		enabled,
	})
}

export async function get_command_history(): Promise<string> {
	return await invoke('plugin:instance|instance_get_command_history')
}

export async function set_command_history(contents: string): Promise<string> {
	return await invoke('plugin:instance|instance_set_command_history', { contents })
}

export async function open_synced_options_folder(): Promise<void> {
	return await invoke('plugin:instance|instance_open_synced_options_folder')
}

export type SyncedServer = {
	id: string
	name: string
	address: string
	accept_textures?: boolean | null
}

export async function list_synced_servers(): Promise<SyncedServer[]> {
	return await invoke('plugin:instance|instance_list_synced_servers')
}

export async function update_synced_server(server: SyncedServer): Promise<void> {
	return await invoke('plugin:instance|instance_update_synced_server', { server })
}

export async function remove_synced_server(serverId: string): Promise<void> {
	return await invoke('plugin:instance|instance_remove_synced_server', { serverId })
}

export async function rebuild_synced_options(instanceId?: string): Promise<void> {
	return await invoke('plugin:instance|instance_rebuild_synced_options', { instanceId })
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

export async function set_project_locked(
	instanceId: string,
	projectPath: string,
	locked: boolean,
): Promise<void> {
	return await invoke('plugin:instance|instance_set_project_locked', {
		instanceId,
		projectPath,
		locked,
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
// included_overrides and excluded_overrides are inherited path rules for files in the export.
// Version id is optional (ie: 1.1.5)
export async function export_instance_mrpack(
	instanceId: string,
	exportLocation: string,
	includedOverrides: string[],
	excludedOverrides: string[],
	versionId?: string,
	description?: string,
	name?: string,
): Promise<void> {
	return await invoke('plugin:instance|instance_export_mrpack', {
		instanceId,
		exportLocation,
		includedOverrides,
		excludedOverrides,
		versionId,
		description,
		name,
	})
}

export type PackExportCandidate = {
	path: string
	type: 'directory' | 'file'
	size?: number
	modified?: number
	count?: number
	disabled: boolean
	defaultSelected: boolean
}

// Given a folder path, populate an array of exportable direct children.
// Allows selection for 'included_overrides' in export_instance_mrpack.
export async function get_pack_export_candidates(
	instanceId: string,
	parent?: string,
): Promise<PackExportCandidate[]> {
	return await invoke('plugin:instance|instance_get_pack_export_candidates', {
		instanceId,
		parent: parent ?? null,
	})
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

export async function edit_generated_icon(
	instanceId: string,
	config: InstanceIconConfig,
	symbolBytes: number[],
): Promise<string> {
	return await invoke('plugin:instance|instance_edit_generated_icon', {
		instanceId,
		config,
		symbolBytes,
	})
}

export async function edit_generated_icon_if_empty(
	instanceId: string,
	config: InstanceIconConfig,
	symbolBytes: number[],
): Promise<boolean> {
	const result = await invoke<string | null>('plugin:instance|instance_edit_generated_icon', {
		instanceId,
		config,
		symbolBytes,
		onlyIfEmpty: true,
	})
	return result !== null
}

export async function cache_generated_icon(
	config: InstanceIconConfig,
	symbolBytes: number[],
	addToRecents = false,
): Promise<string> {
	return await invoke('plugin:instance|instance_cache_generated_icon', {
		config,
		symbolBytes,
		addToRecents,
	})
}

export async function get_recent_icon_configs(): Promise<InstanceIconConfig[]> {
	return await invoke('plugin:instance|instance_get_recent_icon_configs')
}

export type SharedInstanceUsers = {
	user_ids: string[]
	users: SharedInstanceUser[]
	tokens: number
}

export type SharedInstanceJoinType = 'owner' | 'invite' | 'link'

export type SharedInstanceUser = {
	id: string
	joined_at?: string | null
	join_type: SharedInstanceJoinType
	last_played?: string | null
}

export interface SharedInstancePublishPreview {
	sharedInstanceId: string
	latestVersion: number
	diffs: SharedInstanceUpdateDiff[]
	configFiles: string[]
}

export interface SharedInstanceInviteLink {
	inviteId: string
	expiresAt: string
	maxUses: number
}

export interface SharedInstanceInvite {
	id: string
	expiration: string
	maxUses: number
	uses: number
}

export async function can_current_user_use_shared_instances(): Promise<boolean> {
	return await invoke('plugin:instance|instance_share_can_current_user_use')
}

export async function get_shared_instance_users(instanceId: string): Promise<SharedInstanceUsers> {
	return await invoke('plugin:instance|instance_share_get_users', { instanceId })
}

export async function invite_shared_instance_users(
	instanceId: string,
	userIds: string[],
): Promise<SharedInstanceUsers> {
	return await invoke('plugin:instance|instance_share_invite_users', { instanceId, userIds })
}

export async function create_shared_instance_invite_link(
	instanceId: string,
	options: {
		maxAgeSeconds?: number
		maxUses?: number
		replaceInviteId?: string
	} = {},
): Promise<SharedInstanceInviteLink> {
	return await invoke('plugin:instance|instance_share_create_invite_link', {
		instanceId,
		...options,
	})
}

export async function get_shared_instance_invites(
	instanceId: string,
): Promise<SharedInstanceInvite[]> {
	return await invoke('plugin:instance|instance_share_get_invites', { instanceId })
}

export async function revoke_shared_instance_invite(
	instanceId: string,
	inviteId: string,
): Promise<void> {
	return await invoke('plugin:instance|instance_share_revoke_invite', { instanceId, inviteId })
}

export async function remove_shared_instance_users(
	instanceId: string,
	userIds: string[],
	hasPendingRecipients: boolean,
): Promise<SharedInstanceUsers> {
	return await invoke('plugin:instance|instance_share_remove_users', {
		instanceId,
		userIds,
		hasPendingRecipients,
	})
}

export async function get_shared_instance_publish_preview(
	instanceId: string,
): Promise<SharedInstancePublishPreview | null> {
	return await invoke('plugin:instance|instance_share_get_publish_preview', { instanceId })
}

export async function publish_shared_instance(
	instanceId: string,
	configPaths: string[],
): Promise<SharedInstanceAttachment> {
	return await invoke('plugin:instance|instance_share_publish', { instanceId, configPaths })
}

export async function unlink_shared_instance(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_share_unlink', { instanceId })
}

export async function unpublish_shared_instance(instanceId: string): Promise<void> {
	return await invoke('plugin:instance|instance_share_unpublish', { instanceId })
}
