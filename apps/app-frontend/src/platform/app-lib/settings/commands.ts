import { invoke } from '@tauri-apps/api/core'

export function getDefaultAppDirectory(): Promise<string> {
	return invoke('plugin:settings|default_app_directory')
}
