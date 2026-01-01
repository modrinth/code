import type { Labrinth } from '@modrinth/api-client'

const CACHE_MAX_AGE = 60 * 10 // 10 minutes

export default defineCachedEventHandler(
	async (event) => {
		const config = useRuntimeConfig(event)
		const apiBaseUrl = config.apiBaseUrl || config.public.apiBaseUrl

		const response = await $fetch<Labrinth.Tags.v2.GameVersion[]>(
			`${apiBaseUrl}tag/game_version`,
		)

		// nitro wont cache if we throw an error
		if (!response || !Array.isArray(response)) {
			throw createError({ statusCode: 502, message: 'Invalid response from API' })
		}

		return response
	},
	{
		maxAge: CACHE_MAX_AGE,
		name: 'game-versions',
		getKey: () => 'game-versions',
	},
)
