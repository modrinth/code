import { useGeneratedState } from '~/composables/generated'
import { useAppQueryClient } from '~/composables/query-client'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useServerModrinthClient } from '~/server/utils/api-client'

// All valid project type URL segments
const PROJECT_TYPES = ['project', 'mod', 'plugin', 'datapack', 'shader', 'resourcepack', 'modpack']

export default defineNuxtRouteMiddleware(async (to) => {
	// Only handle project routes
	if (!to.params.id || !PROJECT_TYPES.includes(to.params.type as string)) {
		return
	}

	const queryClient = useAppQueryClient()
	const authToken = useCookie('auth-token')
	const client = useServerModrinthClient({ authToken: authToken.value || undefined })
	const tags = useGeneratedState()
	const projectId = to.params.id as string

	// SSR timing for debugging (will be sent to client)
	const ssrTiming = useState<{ projectV2?: number; parallelPrefetch?: number }>('ssr-timing', () => ({}))

	try {
		const t0 = Date.now()
		const project = await queryClient.fetchQuery({
			queryKey: ['project', 'v2', projectId],
			queryFn: () => client.labrinth.projects_v2.get(projectId),
			staleTime: 1000 * 60 * 5,
		})
		if (import.meta.server) {
			ssrTiming.value.projectV2 = Date.now() - t0
			console.log(`[${projectId}] project-v2: ${ssrTiming.value.projectV2}ms`)
		}

		// let page handle 404
		if (!project) return

		const id = project.id

		// Cache by both ID and slug for flexible lookups
		if (projectId !== project.slug) {
			queryClient.setQueryData(['project', 'v2', project.slug], project)
		}
		if (projectId !== id) {
			queryClient.setQueryData(['project', 'v2', id], project)
		}

		// Prefetch core project data in parallel using the resolved ID
		// Versions and dependencies are lazy-loaded client-side for performance
		const t1 = Date.now()
		await Promise.all([
			queryClient.prefetchQuery({
				queryKey: ['project', 'v3', id],
				queryFn: () => client.labrinth.projects_v3.get(id),
				staleTime: 1000 * 60 * 5,
			}),
			queryClient.prefetchQuery({
				queryKey: ['project', id, 'members'],
				queryFn: () => client.labrinth.projects_v3.getMembers(id),
				staleTime: 1000 * 60 * 5,
			}),
			project.organization
				? queryClient.prefetchQuery({
						queryKey: ['project', id, 'organization'],
						queryFn: () => client.labrinth.projects_v3.getOrganization(id),
						staleTime: 1000 * 60 * 5,
					})
				: Promise.resolve(),
		])
		if (import.meta.server) {
			ssrTiming.value.parallelPrefetch = Date.now() - t1
			console.log(`[${projectId}] parallel-prefetch: ${ssrTiming.value.parallelPrefetch}ms`)
		}

		// Determine the correct URL type
		const correctType = getProjectTypeForUrlShorthand(
			project.project_type,
			project.loaders,
			tags.value,
		)

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
	}
})
