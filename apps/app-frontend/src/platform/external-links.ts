import { openUrl } from '@tauri-apps/plugin-opener'

export function openExternalUrl(url: string): Promise<void> {
	return openUrl(url)
}
