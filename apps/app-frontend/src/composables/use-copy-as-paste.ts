import { injectModrinthClient } from '@modrinth/ui'

export function useCopyAsPaste() {
	const client = injectModrinthClient()

	return async function copyAsPaste(content: string) {
		let text = content
		try {
			const { key } = await client.paste.pastes_v0.create(content)
			text = client.paste.pastes_v0.getUrl(key)
		} catch (error) {
			console.error('Failed to upload to paste.modrinth.com, copying raw text instead', error)
		}
		await navigator.clipboard.writeText(text)
	}
}
