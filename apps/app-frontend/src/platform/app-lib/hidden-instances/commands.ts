import { invoke } from '@tauri-apps/api/core'

export function getHiddenInstanceIds(): Promise<string[]> {
	return invoke('plugin:instance|instance_get_hidden_ids')
}

export function setInstanceHidden(instanceId: string, hidden: boolean): Promise<void> {
	return invoke('plugin:instance|instance_set_hidden', { instanceId, hidden })
}
