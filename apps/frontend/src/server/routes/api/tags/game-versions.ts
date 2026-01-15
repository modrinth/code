import type { Labrinth } from '@modrinth/api-client'

import { useServerModrinthClient } from '~/server/utils/api-client'

const CACHE_MAX_AGE = 60 * 10 // 10 minutes

export default defineCachedEventHandler(
	async (event) => {
		const client = useServerModrinthClient({ event })

		const response = await client.request<Labrinth.Tags.v2.GameVersion[]>('/tag/game_version', {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})

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
