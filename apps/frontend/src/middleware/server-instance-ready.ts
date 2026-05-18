import { useAppQueryClient } from '~/composables/query-client'
import { createModrinthClient } from '~/helpers/api.ts'

export default defineNuxtRouteMiddleware(async (to) => {
	const serverId = getRouteParam(to.params.id)
	const worldId = getRouteParam(to.params.instance_id)

	if (!serverId || !worldId) return

	if (import.meta.client) startLoading()

	try {
		const auth = await useAuth()
		if (!auth.value.token) return

		const config = useRuntimeConfig()
		const queryClient = useAppQueryClient()
		const client = createModrinthClient(auth, {
			apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
			archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
			rateLimitKey: config.rateLimitKey,
		})

		await queryClient.ensureQueryData({
			queryKey: ['servers', 'v1', 'detail', serverId],
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		})

		const tab = getInstanceTab(to.path)

		if (tab === 'backups') {
			await queryClient.ensureQueryData({
				queryKey: ['backups', 'list', serverId, worldId],
				queryFn: () => client.archon.backups_v1.list(serverId, worldId),
				staleTime: 30_000,
			})
			return
		}

		if (tab === 'files') {
			const path = typeof to.query.path === 'string' ? to.query.path : '/'
			await queryClient.ensureQueryData({
				queryKey: ['files', serverId, path],
				queryFn: () => client.kyros.files_v0.listDirectory(path, 1, 2000),
				staleTime: 30_000,
			})
			return
		}

		const content = await queryClient.ensureQueryData({
			queryKey: ['content', 'list', 'v1', serverId, worldId],
			queryFn: () =>
				client.archon.content_v1.getAddons(serverId, worldId, { from_modpack: false }),
			staleTime: 30_000,
		})

		const modpackProjectId =
			content.modpack?.spec.platform === 'modrinth' ? content.modpack.spec.project_id : null

		if (modpackProjectId) {
			await queryClient.ensureQueryData({
				queryKey: ['labrinth', 'project', modpackProjectId],
				queryFn: () => client.labrinth.projects_v2.get(modpackProjectId),
				staleTime: 30_000,
			})
		}
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	} finally {
		if (import.meta.client) stopLoading()
	}
})

function getRouteParam(param: string | string[] | undefined): string | null {
	if (Array.isArray(param)) return param[0] ?? null
	return param ?? null
}

function getInstanceTab(path: string): 'content' | 'files' | 'backups' {
	const segments = path.split('/').filter(Boolean)
	const lastSegment = segments[segments.length - 1]
	if (lastSegment === 'files') return 'files'
	return lastSegment === 'backups' ? 'backups' : 'content'
}
