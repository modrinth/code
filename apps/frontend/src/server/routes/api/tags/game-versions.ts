import type { Labrinth } from '@modrinth/api-client'

const CACHE_MAX_AGE = 60 * 10 // 10 minutes

export default defineCachedEventHandler(
	async (event) => {
		const config = useRuntimeConfig(event)
		const apiBaseUrl = config.apiBaseUrl || config.public.apiBaseUrl

		return await $fetch<Labrinth.Tags.v2.GameVersion[]>(`${apiBaseUrl}tag/game_version`)
	},
	{
		maxAge: CACHE_MAX_AGE,
		name: 'game-versions',
		getKey: () => 'game-versions',
	},
)
