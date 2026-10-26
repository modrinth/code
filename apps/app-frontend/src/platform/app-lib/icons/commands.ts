import { invoke } from '@tauri-apps/api/core'

/** Downloads a remote icon into the app's icon cache and returns its asset URL. */
export function cacheRemoteIcon(source: string): Promise<string> {
	return invoke('plugin:utils|cache_remote_icon', { source })
}
