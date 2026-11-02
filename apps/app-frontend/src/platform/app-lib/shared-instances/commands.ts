import { invoke } from '@tauri-apps/api/core'

import type { InstanceLoader } from '@/helpers/types'

export type CachedSharedInstanceSummary = {
	shared_instance_id: string
	role: 'owner' | 'member'
	manager_id: string | null
	name: string
	icon_url: string | null
	game_version: string
	loader: InstanceLoader
}

export function unpublishRemoteSharedInstance(sharedInstanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_unpublish_remote', { sharedInstanceId })
}

export function leaveRemoteSharedInstance(sharedInstanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_leave_remote', { sharedInstanceId })
}

export function dismissSharedInstanceRestoredNotice(instanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_dismiss_restored_notice', { instanceId })
}

export function getCachedSharedInstanceLibrary(): Promise<CachedSharedInstanceSummary[]> {
	return invoke('plugin:instance|instance_share_get_cached_library')
}

export function cacheSharedInstanceLibrary(
	instances: CachedSharedInstanceSummary[],
): Promise<void> {
	return invoke('plugin:instance|instance_share_cache_library', { instances })
}
