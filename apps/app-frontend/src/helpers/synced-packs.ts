import type { ContentItem } from '@modrinth/ui'
import { queryOptions } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'

import { adaptContentItems, isSyncedOptionAvailable } from './instance'
import type { DesyncServerMode } from './worlds'

export type SyncedPackType = 'resourcepack' | 'datapack'
export type SyncedPackAction = 'enable' | 'disable'

export function isSyncedPackTypeAvailable(projectType: ContentItem['project_type']): boolean {
	return (
		['resourcepack', 'datapack'].includes(projectType) &&
		isSyncedOptionAvailable(projectType === 'resourcepack' ? 'resource_packs' : 'data_packs')
	)
}

export interface PackSyncPreview {
	pack: ContentItem
	instances: {
		instance_id: string
		name: string
		game_version: string
		compatible: boolean
		participating: boolean
	}[]
}

export const syncedPackKeys = {
	all: ['synced-packs'] as const,
	list: (type: SyncedPackType) => ['synced-packs', 'list', type] as const,
}

export function syncedPackQueryOptions(type: SyncedPackType) {
	return queryOptions({
		queryKey: syncedPackKeys.list(type),
		queryFn: () => list_synced_packs(type),
		enabled: isSyncedPackTypeAvailable(type),
	})
}

export async function list_synced_packs(projectType: SyncedPackType): Promise<ContentItem[]> {
	const items = await invoke<ContentItem[]>('plugin:instance|instance_list_synced_packs', {
		projectType,
	})
	return adaptContentItems(items).map((item) => ({ ...item, external: !item.project }))
}

export function get_pack_sync_preview(
	instanceId: string,
	projectPath: string,
): Promise<PackSyncPreview> {
	return invoke('plugin:instance|instance_get_pack_sync_preview', { instanceId, projectPath })
}

export function sync_pack(instanceId: string, projectPath: string): Promise<void> {
	return invoke('plugin:instance|instance_sync_pack', { instanceId, projectPath })
}

export function desync_pack(
	instanceId: string,
	packId: string,
	mode: DesyncServerMode,
): Promise<void> {
	return invoke('plugin:instance|instance_desync_pack', { instanceId, packId, mode })
}

export function upload_synced_pack(
	path: string,
	projectType: SyncedPackType,
	gameVersions: string[],
): Promise<void> {
	return invoke('plugin:instance|instance_upload_synced_pack', { path, projectType, gameVersions })
}

export function set_synced_pack_enabled(packId: string, enabled: boolean): Promise<void> {
	return invoke('plugin:instance|instance_set_synced_pack_enabled', { packId, enabled })
}

export function remove_synced_pack(packId: string): Promise<void> {
	return invoke('plugin:instance|instance_remove_synced_pack', { packId })
}
