import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'
import { useServerModrinthClient } from '~/server/utils/api-client'

// All valid project type URL segments
const PROJECT_TYPES = ['project', 'mod', 'plugin', 'datapack', 'shader', 'resourcepack', 'modpack']

export default defineNuxtRouteMiddleware(async (to) => {
	// Only handle project routes
	if (!to.params.id || !PROJECT_TYPES.includes(to.params.type as string)) {
		return
	}

	const authToken = useCookie('auth-token')
	const client = useServerModrinthClient({ authToken: authToken.value || undefined })
	const tags = useGeneratedState()

	try {
		const project = await client.labrinth.projects_v2.get(to.params.id as string)

		if (!project) {
			return
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
