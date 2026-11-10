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

export type SharedInstanceRelinkBlocker =
	| { type: 'not_owner' }
	| { type: 'server_managed' }
	| { type: 'already_linked'; instanceId: string; instanceName: string }

export type SharedInstanceRelinkPreview = {
	sharedInstanceId: string
	name: string
	iconUrl: string | null
	version: number
	gameVersion: string
	loader: InstanceLoader
	gameVersionMismatch: boolean
	loaderMismatch: boolean
	blocker: SharedInstanceRelinkBlocker | null
}

/** Resolves a pasted invite link or ID to the shared instance an instance would be relinked to. */
export function getSharedInstanceRelinkPreview(
	instanceId: string,
	input: string,
): Promise<SharedInstanceRelinkPreview> {
	return invoke('plugin:instance|instance_share_get_relink_preview', { instanceId, input })
}

/** Links an instance back to a shared instance the current user owns. */
export function relinkSharedInstance(instanceId: string, sharedInstanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_relink', { instanceId, sharedInstanceId })
}

export function unpublishRemoteSharedInstance(sharedInstanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_unpublish_remote', { sharedInstanceId })
}

export function leaveRemoteSharedInstance(sharedInstanceId: string): Promise<void> {
	return invoke('plugin:instance|instance_share_leave_remote', { sharedInstanceId })
}

export type SharedInstanceLockStatus = {
	locked: boolean
	reason: string | null
}

export function refreshSharedInstanceLock(instanceId: string): Promise<SharedInstanceLockStatus> {
	return invoke('plugin:instance|instance_share_refresh_lock', { instanceId })
}

export function refreshSharedInstanceLocks(): Promise<void> {
	return invoke('plugin:instance|instance_share_refresh_locks')
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
