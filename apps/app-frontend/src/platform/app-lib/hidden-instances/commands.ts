import { invoke } from '@tauri-apps/api/core'

/** Returns the IDs of library instances the signed-in user has hidden on this device, most recent first. */
export function getHiddenInstanceIds(): Promise<string[]> {
	return invoke('plugin:instance|instance_get_hidden_ids')
}

/** Hides or shows a library instance for the signed-in user on this device. */
export function setInstanceHidden(instanceId: string, hidden: boolean): Promise<void> {
	return invoke('plugin:instance|instance_set_hidden', { instanceId, hidden })
}
