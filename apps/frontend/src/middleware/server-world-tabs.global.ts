import { createModrinthClient } from '~/helpers/api.ts'

export default defineNuxtRouteMiddleware(async (to) => {
	const match = to.path.match(/^\/hosting\/manage\/([^/]+)\/(content|files|backups)\/?$/)
	if (!match) return

	const serverId = decodeURIComponent(match[1])
	const tab = match[2]
	const worldsPath = `/hosting/manage/${encodeURIComponent(serverId)}/worlds`
	const tabPath = tab === 'content' ? '' : `/${tab}`
	const auth = await useAuth()

	if (auth.value.token) {
		try {
			const config = useRuntimeConfig()
			const client = createModrinthClient(auth, {
				apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
				archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
				rateLimitKey: config.rateLimitKey,
			})
			const serverFull = await client.archon.servers_v1.get(serverId)
			const world = serverFull.worlds.find((item) => item.is_active) ?? serverFull.worlds[0]

			if (world) {
				return navigateTo(
					{
						path: `${worldsPath}/${encodeURIComponent(world.id)}${tabPath}`,
						query: to.query,
						hash: to.hash,
					},
					{ replace: true },
				)
			}
		} catch {
			return navigateTo({ path: worldsPath, query: to.query, hash: to.hash }, { replace: true })
		}
	}

	return navigateTo({ path: worldsPath, query: to.query, hash: to.hash }, { replace: true })
})
