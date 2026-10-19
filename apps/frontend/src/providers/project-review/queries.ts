import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { projectQueryOptions } from '~/composables/queries/project'

export const reviewQueryOptions = {
	thread: (threadId: string, client: AbstractModrinthClient) => ({
		queryKey: ['thread', threadId] as const,
		queryFn: () => client.labrinth.threads_v3.getThread(threadId),
	}),
	report: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['tech-review-project-report', projectId] as const,
		queryFn: () => client.labrinth.tech_review_internal.getProjectReport(projectId),
	}),
	versions: (id: string, client: AbstractModrinthClient, queryClient: QueryClient) => {
		const options = projectQueryOptions.versionsV3(id, client)
		return {
			...options,
			queryFn: async () => {
				const [versions, project] = await Promise.all([
					options.queryFn(),
					queryClient.fetchQuery(projectQueryOptions.v3(id, client)),
				])
				const loadedIds = new Set(versions.map((version) => version.id))
				const missingIds = project.versions.filter((versionId) => !loadedIds.has(versionId))
				/** The list endpoint can transiently omit versions that are still available by ID. */
				for (let index = 0; index < missingIds.length; index += 100) {
					const recovered = await client.labrinth.versions_v3.getVersions(
						missingIds.slice(index, index + 100),
					)
					for (const version of recovered) {
						if (loadedIds.has(version.id)) continue
						loadedIds.add(version.id)
						versions.push(version)
					}
				}
				if (missingIds.some((versionId) => !loadedIds.has(versionId))) {
					throw new Error('Could not load all project versions')
				}
				return versions
			},
		}
	},
}
