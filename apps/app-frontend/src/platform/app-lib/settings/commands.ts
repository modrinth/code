import { invoke } from '@tauri-apps/api/core'

export function getDefaultAppDirectory(): Promise<string> {
	return invoke('plugin:settings|default_app_directory')
}

/** Replaces the analytics installation ID with a new random one and returns it. */
export function resetAnalyticsInstallationId(): Promise<string> {
	return invoke('plugin:settings|reset_analytics_installation_id')
}
