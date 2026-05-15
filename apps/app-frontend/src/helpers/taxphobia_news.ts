import { invoke } from '@tauri-apps/api/core'

type TaxphobiaNewsItem = {
	id: string
	title: string
	content: string
	excerpt?: string | null
	date: string
	type: string
	pinned: boolean
	image?: string | null
	url?: string | null
}

type TaxphobiaNewsResponse = {
	success: boolean
	news: TaxphobiaNewsItem[]
}

export async function get_taxphobia_news(): Promise<TaxphobiaNewsItem[]> {
	try {
		const result = await invoke<TaxphobiaNewsResponse>('get_taxphobia_news')
		if (result?.success && Array.isArray(result.news)) {
			return result.news
		}
		return []
	} catch (error) {
		console.error('Failed to fetch TaxPhobia news:', error)
		return []
	}
}
