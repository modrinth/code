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
		// Step 1: Fetch V2 project to get the actual project ID
		const project = await queryClient.fetchQuery({
			queryKey: ['project', 'v2', projectId],
			queryFn: () => client.labrinth.projects_v2.get(projectId),
			staleTime: 1000 * 60 * 5,
		})

		if (!project) return

		const id = project.id

		// Cache by both ID and slug for flexible lookups
		if (projectId !== project.slug) {
			queryClient.setQueryData(['project', 'v2', project.slug], project)
		}
		if (projectId !== id) {
			queryClient.setQueryData(['project', 'v2', id], project)
		}

		// Prefetch all dependent data in parallel using the resolved ID
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
			queryClient.prefetchQuery({
				queryKey: ['project', id, 'dependencies'],
				queryFn: () => client.labrinth.projects_v2.getDependencies(id),
				staleTime: 1000 * 60 * 5,
			}),
			queryClient.prefetchQuery({
				queryKey: ['project', id, 'versions', 'v2'],
				queryFn: () =>
					client.labrinth.versions_v3.getProjectVersions(id, { include_changelog: false }),
				staleTime: 1000 * 60 * 5,
			}),
			queryClient.prefetchQuery({
				queryKey: ['project', id, 'versions', 'v3'],
				queryFn: () =>
					client.labrinth.versions_v3.getProjectVersions(id, {
						include_changelog: false,
						apiVersion: 3,
					}),
				staleTime: 1000 * 60 * 5,
			}),
			queryClient.prefetchQuery({
				queryKey: ['project', id, 'organization'],
				queryFn: () => client.labrinth.projects_v3.getOrganization(id),
				staleTime: 1000 * 60 * 5,
			}),
		])

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
