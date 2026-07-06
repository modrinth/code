import { useGeneratedState } from '~/composables/generated'
import { projectQueryOptions } from '~/composables/queries/project'
import { useAppQueryClient } from '~/composables/query-client'
import { createModrinthClient } from '~/helpers/api.ts'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useServerModrinthClient } from '~/server/utils/api-client'

// All valid project type URL segments
const PROJECT_TYPES = [
	'project',
	'mod',
	'plugin',
	'datapack',
	'shader',
	'resourcepack',
	'modpack',
	'server',
	'minecraft_java_server',
]

export default defineNuxtRouteMiddleware(async (to) => {
	const routeProjectParam = to.params.project
	const projectId = Array.isArray(routeProjectParam) ? routeProjectParam[0] : routeProjectParam
	const routeType = Array.isArray(to.params.type) ? to.params.type[0] : to.params.type

	// Only handle project routes
	if (!projectId || !routeType || !PROJECT_TYPES.includes(routeType)) {
		return
	}

	const queryClient = useAppQueryClient()
	const client = await getProjectMiddlewareClient()
	const tags = useGeneratedState()

	if (import.meta.client) startLoading()

	try {
		// Fetch v2 and v3 in parallel — cache both for the page's useQuery calls
		const [project, projectV3] = await Promise.all([
			queryClient.fetchQuery(projectQueryOptions.v2(projectId, client)),
			queryClient.fetchQuery(projectQueryOptions.v3(projectId, client)),
		])

		// Let page handle 404
		if (!project) return

		// Cache by slug if we looked up by ID (or vice versa)
		if (projectId !== project.slug) {
			queryClient.setQueryData(['project', 'v2', project.slug], project)
			queryClient.setQueryData(['project', 'v3', project.slug], projectV3)
		}
		if (projectId !== project.id) {
			queryClient.setQueryData(['project', 'v2', project.id], project)
			queryClient.setQueryData(['project', 'v3', project.id], projectV3)
		}

		const projectType = projectV3.minecraft_server != null ? 'server' : project.project_type
		// Determine the correct URL type
		const correctType = getProjectTypeForUrlShorthand(projectType, project.loaders, tags.value)

		// Preserve the rest of the path (subpages like /versions, /settings, etc.)
		const pathParts = to.path.split('/')
		pathParts.splice(0, 3) // Remove '', type, and id
		const remainder = pathParts.filter((x) => x).join('/')

		// Build the canonical path
		const canonicalPath = `/${correctType}/${project.slug}${remainder ? `/${remainder}` : ''}`

		// Only redirect if the path actually changed
		if (to.path !== canonicalPath) {
			return navigateTo(
				{
					path: canonicalPath,
					query: to.query,
					hash: to.hash,
				},
				{
					redirectCode: 301,
					replace: true,
				},
			)
		}
	} catch {
		// Let the page handle 404s and other errors
	} finally {
		if (import.meta.client) stopLoading()
	}
})

async function getProjectMiddlewareClient() {
	if (import.meta.server) {
		const authToken = useCookie('auth-token')
		return useServerModrinthClient({ authToken: authToken.value || undefined })
	}

	const auth = await useAuth()
	const config = useRuntimeConfig()

	return createModrinthClient(auth, {
		apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
		archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
		rateLimitKey: config.rateLimitKey,
	})
}
