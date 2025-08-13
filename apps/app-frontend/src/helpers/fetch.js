import { injectNotificationManager } from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { fetch } from '@tauri-apps/plugin-http'

export const useFetch = async (url, item, isSilent) => {
	try {
		const version = await getVersion()
		return await fetch(url, {
			method: 'GET',
			headers: { 'User-Agent': `modrinth/theseus/${version} (support@modrinth.com)` },
		})
	} catch (err) {
		if (!isSilent) {
			const { handleError } = injectNotificationManager()
			handleError({ message: `Error fetching ${item}` })
		}
		console.error(err)
	}
}
