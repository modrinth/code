import { projectQueryOptions } from '~/composables/queries/project.ts'
import { versionQueryOptions } from '~/composables/queries/version.ts'
import { useAppQueryClient } from '~/composables/query-client.ts'
import { getProjectTypeForUrl } from '~/helpers/projects'
import { getProjectMiddlewareClient } from '~/middleware/project.global.ts'

export default defineNuxtRouteMiddleware(async (to) => {
	const match = to.path.match(/^\/version\/(\w+)\/?$/)
	if (!match) return

	const versionId = match[1]

	try {
		const client = await getProjectMiddlewareClient(to)
		const queryClient = useAppQueryClient()

		const version = await queryClient.fetchQuery(versionQueryOptions.v3(versionId, client))

		if (version) {
			const project = await queryClient.fetchQuery(
				projectQueryOptions.v3(version.project_id, client),
			)
			const type =
				project.minecraft_server == null
					? getProjectTypeForUrl(project.project_types[0], project.loaders)
					: 'server'

			if (project) {
				return navigateTo(
					{
						name: 'type-project-version-version',
						params: {
							type: type,
							project: project.slug ? project.slug : project.id,
							version: version.id,
						},
					},
					{
						redirectCode: 302,
						replace: true,
					},
				)
			}
		}
	} catch {
		// Project or Version not found moment
	}

	return createError({ fatal: true, statusCode: 404, statusMessage: 'Version not found' })
})
