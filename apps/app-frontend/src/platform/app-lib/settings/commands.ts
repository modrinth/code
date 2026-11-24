import { invoke } from '@tauri-apps/api/core'

export function getDefaultAppDirectory(): Promise<string> {
	return invoke('plugin:settings|default_app_directory')
}

export function getSandboxUnavailableReason(): Promise<string | null> {
	return invoke('plugin:settings|settings_sandbox_unavailable_reason')
}
