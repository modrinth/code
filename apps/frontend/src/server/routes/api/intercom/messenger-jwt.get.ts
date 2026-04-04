import { type Labrinth, ModrinthApiError } from '@modrinth/api-client'
import { SignJWT } from 'jose'

import { useServerModrinthClient } from '~/server/utils/api-client'

type IntercomTokenResponse = {
	token: string
}

async function signIntercomUserJwt(
	user: { id: string; username: string; email?: string; created: string },
	secret: string,
): Promise<string> {
	const createdAt = Math.floor(new Date(user.created).getTime() / 1000)

	const payload: Record<string, string | number> = {
		user_id: user.id,
		name: user.username,
	}

	if (user.email) {
		payload.email = user.email
	}

	if (Number.isFinite(createdAt)) {
		payload.created_at = createdAt
	}

	return await new SignJWT(payload)
		.setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
		.setIssuedAt()
		.setExpirationTime('1h')
		.sign(new TextEncoder().encode(secret))
}

export default defineEventHandler(async (event): Promise<IntercomTokenResponse> => {
	if (event.method !== 'GET') {
		throw createError({
			statusCode: 405,
			message: 'Method not allowed',
		})
	}

	const authToken = getCookie(event, 'auth-token')
	if (!authToken) {
		throw createError({
			statusCode: 401,
			message: 'Authentication required',
		})
	}

	setHeader(event, 'cache-control', 'private, no-store, max-age=0')

	const config = useRuntimeConfig(event)
	if (!config.intercomIdentitySecret) {
		throw createError({
			statusCode: 500,
			message: 'Intercom identity secret is not configured',
		})
	}

	const client = useServerModrinthClient({
		event,
		authToken,
	})

	let user: { id: string; username: string; email?: string; created: string }
	try {
		const currentUser = await client.request<Labrinth.Users.v2.User>('/user', {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
		user = {
			id: currentUser.id,
			username: currentUser.username,
			email: currentUser.email,
			created: currentUser.created,
		}
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 401) {
			throw createError({
				statusCode: 401,
				message: 'Authentication required',
			})
		}

		throw createError({
			statusCode: 502,
			message: 'Failed to resolve current user',
		})
	}

	const token = await signIntercomUserJwt(user, config.intercomIdentitySecret)

	return {
		token,
	}
})
