import { invoke } from '@tauri-apps/api/core'

export const FAVORITES_GROUP_ID = 'group:favorites'
export const MAX_INSTANCE_GROUP_NAME_LENGTH = 256

export type InstanceGroupDefinition = {
	id: string
	name: string
}

export type InstanceGroupMembershipUpdate = {
	instance_id: string
	group_ids: string[]
}

export async function list_groups(): Promise<InstanceGroupDefinition[]> {
	return await invoke('plugin:instance|instance_list_groups')
}

export async function create_group(name: string): Promise<InstanceGroupDefinition> {
	return await invoke('plugin:instance|instance_create_group', { name })
}

export async function rename_group(id: string, newName: string): Promise<InstanceGroupDefinition> {
	return await invoke('plugin:instance|instance_rename_group', { id, newName })
}

export async function delete_group(id: string): Promise<void> {
	return await invoke('plugin:instance|instance_delete_group', { id })
}

export async function set_group_order(groupIds: string[]): Promise<void> {
	return await invoke('plugin:instance|instance_set_group_order', { groupIds })
}

export async function set_group_memberships(
	updates: InstanceGroupMembershipUpdate[],
): Promise<void> {
	return await invoke('plugin:instance|instance_set_group_memberships', { updates })
}
