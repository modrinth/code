import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { projectQueryOptions } from '~/composables/queries/project'

export type QueueProjectPreview = Pick<Labrinth.Projects.v3.Project, 'id' | 'name' | 'icon_url'>

export const queueProjectPreviewKey = (id: string) => ['project', 'queue-preview', id] as const

function preview(project: QueueProjectPreview): QueueProjectPreview {
	return { id: project.id, name: project.name, icon_url: project.icon_url }
}

function cachedProject(queryClient: QueryClient, id: string) {
	return (
		queryClient.getQueryData<Labrinth.Projects.v3.Project>(['project', 'v3', id]) ??
		queryClient.getQueryData<Labrinth.Projects.v2.Project>(['project', 'v2', id])
	)
}

export function seedQueueProjectPreviews(queryClient: QueryClient, ids: string[]) {
	const wanted = new Set(ids)
	const cachedQueues = queryClient.getQueriesData<Labrinth.Moderation.Internal.ProjectsResponse>({
		queryKey: ['moderation-projects'],
	})
	for (const [, response] of cachedQueues) {
		for (const project of response?.projects ?? []) {
			if (wanted.has(project.id)) {
				queryClient.setQueryData(queueProjectPreviewKey(project.id), preview(project))
			}
		}
	}
	for (const id of ids) {
		const project = cachedProject(queryClient, id)
		if (project) queryClient.setQueryData(queueProjectPreviewKey(id), preview(project))
	}
}

export function queueProjectPreviewOptions(
	id: string,
	client: AbstractModrinthClient,
	queryClient: QueryClient,
) {
	return {
		queryKey: queueProjectPreviewKey(id),
		queryFn: async () =>
			preview(
				cachedProject(queryClient, id) ??
					(await queryClient.fetchQuery(projectQueryOptions.v3(id, client))),
			),
		staleTime: Infinity,
		gcTime: 1000 * 60 * 60,
	}
}
