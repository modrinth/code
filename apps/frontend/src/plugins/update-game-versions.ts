import type { Labrinth } from '@icarus/api-client'

export default defineNuxtPlugin(async () => {
	try {
		const gameVersions = await $fetch<Labrinth.Tags.v2.GameVersion[]>('/api/tags/game-versions')

		if (gameVersions && gameVersions.length > 0) {
			const state = useState<{ gameVersions: Labrinth.Tags.v2.GameVersion[] }>('generatedState')

			if (state.value) {
				state.value.gameVersions = gameVersions
			}
		}
	} catch (error) {
		console.error('[Game Version Updater] Failed to fetch:', error)
	}
})
