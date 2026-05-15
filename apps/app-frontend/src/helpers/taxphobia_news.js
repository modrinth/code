import { invoke } from '@tauri-apps/api/core'

export async function get_taxphobia_news() {
	try {
		const result = await invoke('plugin:taxphobia-news|get_taxphobia_news')
		if (result?.success && result.news) {
			return result.news
		}
		return []
	} catch (error) {
		console.error('Failed to fetch Taxphobia news:', error)
		return []
	}
}
