import { invoke } from '@tauri-apps/api/core'

export const FAVORITES_GROUP_ID = 'group:favorites'

export type InstanceGroupDefinition = {
	id: string
	name: string
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
