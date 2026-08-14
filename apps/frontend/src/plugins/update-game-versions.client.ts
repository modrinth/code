import type { Labrinth } from '@modrinth/api-client'

import { setGameVersions } from '~/composables/generated'

export default defineNuxtPlugin((nuxtApp) => {
	// Deferred until after hydration: the server renders with the build-time game versions, so
	// swapping them in during setup would make the first client render disagree with the markup.
	nuxtApp.hook('app:suspense:resolve', async () => {
		try {
			const gameVersions = await $fetch<Labrinth.Tags.v2.GameVersion[]>('/api/tags/game-versions')

			if (gameVersions && gameVersions.length > 0) {
				setGameVersions(gameVersions)
			}
		} catch (error) {
			console.error('[Game Version Updater] Failed to fetch:', error)
		}
	})
})
