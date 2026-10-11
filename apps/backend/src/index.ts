import { handleAccounts } from './accounts'
import { ApiError, json } from './common/http'
import type { Env } from './common/types'
import { handleCores } from './cores'
import { handleSharing } from './sharing'
import { handleSocial } from './social'
import { handleFriendsSocket } from './social/socket'
export { FriendsHub } from './social/socket'

function corsOrigin(request: Request, env: Env): string | null {
	const origin = request.headers.get('Origin')
	if (!origin) return null
	if ((env.CORS_ORIGINS ?? '').split(',').includes(origin)) return origin
	if (env.LOCAL_DEV === 'true') {
		try {
			const parsed = new URL(origin)
			if (
				['localhost', '127.0.0.1', '[::1]', 'tauri.localhost'].includes(parsed.hostname) ||
				origin === 'tauri://localhost'
			)
				return origin
		} catch {
			return null
		}
	}
	return null
}

export default {
	async fetch(request: Request, env: Env): Promise<Response> {
		const origin = corsOrigin(request, env)
		let response: Response
		try {
			if (request.method === 'OPTIONS') response = new Response(null, { status: 204 })
			else if (new URL(request.url).pathname === '/health') {
				await env.DB.prepare('SELECT 1 FROM users LIMIT 1').first()
				response = json({ status: 'ok' })
			} else
				response =
					(await handleFriendsSocket(request, env)) ??
					(await handleSharing(request, env)) ??
					(await handleCores(request, env)) ??
					(await handleAccounts(request, env)) ??
					(await handleSocial(request, env)) ??
					json({ error: 'not_found', description: 'Not found' }, 404)
		} catch (error) {
			if (error instanceof ApiError)
				response = json({ error: error.error, description: error.description }, error.status)
			else {
				console.error('Backend request failed', error)
				response = json({ error: 'internal_error', description: 'An internal error occurred' }, 500)
			}
		}
		if (response.status === 101) return response
		if (origin) {
			response.headers.set('Access-Control-Allow-Origin', origin)
			response.headers.set('Vary', 'Origin')
			response.headers.set('Access-Control-Allow-Methods', 'GET, POST, PUT, PATCH, DELETE, OPTIONS')
			response.headers.set(
				'Access-Control-Allow-Headers',
				'Authorization, Content-Type, Range, X-Local-Dev-Secret, X-Requested-With, X-Panel-Version',
			)
			response.headers.set('Access-Control-Expose-Headers', 'Content-Range, Accept-Ranges, ETag')
		}
		return response
	},
} satisfies ExportedHandler<Env>
