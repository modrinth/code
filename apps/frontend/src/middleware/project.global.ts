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

	try {
		// Fetch v2 project for redirect check AND cache it for the page
		// Using fetchQuery ensures the page's useQuery gets this cached result
		const project = await queryClient.fetchQuery({
			queryKey: ['project', 'v2', projectId],
			queryFn: () => client.labrinth.projects_v2.get(projectId),
			staleTime: 1000 * 60 * 5,
		})

		// Let page handle 404
		if (!project) return

		// Cache by slug if we looked up by ID (or vice versa)
		if (projectId !== project.slug) {
			queryClient.setQueryData(['project', 'v2', project.slug], project)
		}
		if (projectId !== project.id) {
			queryClient.setQueryData(['project', 'v2', project.id], project)
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
