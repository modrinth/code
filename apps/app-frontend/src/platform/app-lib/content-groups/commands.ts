import { invoke } from '@tauri-apps/api/core'

import type { InstanceContentGroup } from './types'

export function listContentGroups(instanceId: string): Promise<InstanceContentGroup[]> {
	return invoke('plugin:instance|instance_list_content_groups', { instanceId })
}

export function createContentGroup(
	instanceId: string,
	contentType: string,
	name: string,
	itemKeys: string[],
): Promise<InstanceContentGroup> {
	return invoke('plugin:instance|instance_create_content_group', {
		instanceId,
		contentType,
		name,
		itemKeys,
	})
}

export function renameContentGroup(id: string, newName: string): Promise<void> {
	return invoke('plugin:instance|instance_rename_content_group', { id, newName })
}

export function deleteContentGroup(id: string): Promise<void> {
	return invoke('plugin:instance|instance_delete_content_group', { id })
}

export function setContentGroupOrder(
	instanceId: string,
	contentType: string,
	groupIds: string[],
): Promise<void> {
	return invoke('plugin:instance|instance_set_content_group_order', {
		instanceId,
		contentType,
		groupIds,
	})
}

export function setContentGroupMemberships(
	instanceId: string,
	contentType: string,
	itemKeys: string[],
	groupId: string | null,
): Promise<void> {
	return invoke('plugin:instance|instance_set_content_group_memberships', {
		instanceId,
		contentType,
		itemKeys,
		groupId,
	})
}
