import { createHmac } from 'node:crypto'

import { ModrinthApiError } from '@modrinth/api-client'

import { useServerModrinthClient } from '~/server/utils/api-client'

type AuthenticatedUser = {
	id: string
	username: string
	email?: string
	created: string
}

type IntercomTokenResponse = {
	token: string
	user: AuthenticatedUser
}

function base64UrlEncode(input: string | Buffer): string {
	return Buffer.from(input)
		.toString('base64')
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/g, '')
}

function signIntercomUserJwt(payload: Record<string, unknown>, secret: string): string {
	const header = {
		alg: 'HS256',
		typ: 'JWT',
	}

	const encodedHeader = base64UrlEncode(JSON.stringify(header))
	const encodedPayload = base64UrlEncode(JSON.stringify(payload))
	const unsignedToken = `${encodedHeader}.${encodedPayload}`

	const signature = createHmac('sha256', secret).update(unsignedToken).digest()

	return `${unsignedToken}.${base64UrlEncode(signature)}`
}

export default defineEventHandler(async (event): Promise<IntercomTokenResponse> => {
	if (getMethod(event) !== 'GET') {
		throw createError({
			statusCode: 405,
			message: 'Method not allowed',
		})
	}

	const headerToken = getHeader(event, 'authorization')
	const parsedHeaderToken = headerToken?.replace(/^Bearer\s+/i, '').trim()
	const authToken = parsedHeaderToken || getCookie(event, 'auth-token')
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

	let user: AuthenticatedUser
	try {
		const currentUser = await client.request<AuthenticatedUser>('/user', {
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

	const now = Math.floor(Date.now() / 1000)
	const token = signIntercomUserJwt(
		{
			user_id: user.id,
			email: user.email,
			name: user.username,
			created_at: Math.floor(new Date(user.created).getTime() / 1000),
			iat: now,
			exp: now + 60 * 60,
		},
		config.intercomIdentitySecret,
	)

	return {
		token,
		user,
	}
})
