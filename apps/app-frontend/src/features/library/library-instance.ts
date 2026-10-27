import type { GameInstance } from '@/helpers/types'

import type { UninstalledSharedInstance } from './shared-instances/types'

export type LibraryInstance = GameInstance | UninstalledSharedInstance

export type SharedInstanceOwner = {
	name: string
	avatarUrl: string | null
}

export const isUninstalledSharedInstance = (
	instance: LibraryInstance,
): instance is UninstalledSharedInstance =>
	'type' in instance && instance.type === 'uninstalled_shared_instance'

export const isOwnedByOthers = (instance: LibraryInstance) =>
	isUninstalledSharedInstance(instance)
		? instance.role !== 'owner'
		: instance.shared_instance?.role === 'member'

export const getSharedInstanceManagerId = (instance: LibraryInstance) => {
	if (isUninstalledSharedInstance(instance)) return instance.manager_id
	if (instance.shared_instance?.server_manager_name) return null
	return instance.shared_instance?.manager_id ?? null
}
