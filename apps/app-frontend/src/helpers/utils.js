import { invoke } from '@tauri-apps/api/core'

import { get_full_path, get_mod_full_path } from '@/helpers/profile'

export async function isDev() {
	return await invoke('is_dev')
}

// One of 'Windows', 'Linux', 'MacOS'
export async function getOS() {
	return await invoke('plugin:utils|get_os')
}

export async function openPath(path) {
	return await invoke('plugin:utils|open_path', { path })
}

export async function highlightInFolder(path) {
	return await invoke('plugin:utils|highlight_in_folder', { path })
}

export async function showLauncherLogsFolder() {
	return await invoke('plugin:utils|show_launcher_logs_folder', {})
}

// Opens a profile's folder in the OS file explorer
export async function showProfileInFolder(path) {
	const fullPath = await get_full_path(path)
	return await openPath(fullPath)
}

export async function highlightModInProfile(profilePath, projectPath) {
	const fullPath = await get_mod_full_path(profilePath, projectPath)
	return await highlightInFolder(fullPath)
}

export async function restartApp() {
	return await invoke('restart_app')
}

/**
 * @deprecated This method is no longer needed, and just returns its parameter
 */
export function sanitizePotentialFileUrl(url) {
	return url
}

export const releaseColor = (releaseType) => {
	switch (releaseType) {
		case 'release':
			return 'green'
		case 'beta':
			return 'orange'
		case 'alpha':
			return 'red'
		default:
			return ''
	}
}

export async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
}
