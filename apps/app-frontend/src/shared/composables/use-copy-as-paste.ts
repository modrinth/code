import { injectModrinthClient } from '@modrinth/ui'

export function useCopyAsPaste() {
	const client = injectModrinthClient()

	return async function copyAsPaste(content: string): Promise<string | null> {
		let url: string | null = null
		try {
			const { key } = await client.paste.pastes_v0.create(content, 'log')
			url = client.paste.pastes_v0.getUrl(key)
		} catch (error) {
			console.error('Failed to upload to paste.modrinth.com, copying raw text instead', error)
		}
		await navigator.clipboard.writeText(url ?? content)
		return url
	}
}
