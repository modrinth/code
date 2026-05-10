import { type Labrinth, ModrinthApiError } from '@modrinth/api-client'

import { useServerModrinthClient } from '~/server/utils/api-client'

export default defineEventHandler(async (event) => {
	const username = getRouterParam(event, 'username')

	if (!username) {
		throw createError({
			statusCode: 400,
			message: 'Missing username',
		})
	}

	const client = useServerModrinthClient({ event })

	let user: Labrinth.Users.v2.User
	try {
		user = await client.labrinth.users_v2.get(username)
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) {
			throw createError({
				statusCode: 404,
				message: 'User not found',
			})
		}

		throw createError({
			statusCode: 502,
			message: 'Failed to resolve user avatar',
		})
	}

	if (!user.avatar_url) {
		throw createError({
			statusCode: 404,
			message: 'User avatar not found',
		})
	}

	setHeader(event, 'cache-control', 'public, max-age=300, s-maxage=600')
	return sendRedirect(event, user.avatar_url, 302)
})
