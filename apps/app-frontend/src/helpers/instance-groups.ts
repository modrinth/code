import { invoke } from '@tauri-apps/api/core'

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

export async function rename_group(
	oldName: string,
	newName: string,
): Promise<InstanceGroupDefinition> {
	return await invoke('plugin:instance|instance_rename_group', { oldName, newName })
}

export async function delete_group(name: string): Promise<void> {
	return await invoke('plugin:instance|instance_delete_group', { name })
}
