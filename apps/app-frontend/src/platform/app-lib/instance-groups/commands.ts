import { invoke } from '@tauri-apps/api/core'

import type { InstanceGroupDefinition, InstanceGroupMembershipUpdate } from './types'

export function listGroups(): Promise<InstanceGroupDefinition[]> {
	return invoke('plugin:instance|instance_list_groups')
}

export function createGroup(name: string): Promise<InstanceGroupDefinition> {
	return invoke('plugin:instance|instance_create_group', { name })
}

export function renameGroup(id: string, newName: string): Promise<InstanceGroupDefinition> {
	return invoke('plugin:instance|instance_rename_group', { id, newName })
}

export function deleteGroup(id: string): Promise<void> {
	return invoke('plugin:instance|instance_delete_group', { id })
}

export function setGroupOrder(groupIds: string[]): Promise<void> {
	return invoke('plugin:instance|instance_set_group_order', { groupIds })
}

export function setGroupMemberships(updates: InstanceGroupMembershipUpdate[]): Promise<void> {
	return invoke('plugin:instance|instance_set_group_memberships', { updates })
}
