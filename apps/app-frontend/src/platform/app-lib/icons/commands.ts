import { invoke } from '@tauri-apps/api/core'

export function cacheRemoteIcon(source: string): Promise<string> {
	return invoke('plugin:utils|cache_remote_icon', { source })
}
