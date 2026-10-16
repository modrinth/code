import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { creatorProjectStatsQueryOptions } from '~/composables/creator-project-stats'
import { projectQueryOptions, warmProjectCheckCaches } from '~/composables/queries/project'

import { reviewQueryOptions } from './queries'

export async function prefetchReviewProject(
	id: string,
	client: AbstractModrinthClient,
	queryClient: QueryClient,
	isCurrent: () => boolean,
) {
	if (!isCurrent()) return
	const projectPromise = queryClient.fetchQuery(projectQueryOptions.v3(id, client))
	const membersPromise = queryClient.fetchQuery(projectQueryOptions.members(id, client))
	const disclosuresPromise = queryClient.fetchQuery(projectQueryOptions.disclosures(id, client))

	await Promise.allSettled([
		queryClient.prefetchQuery(projectQueryOptions.v2(id, client)),
		queryClient.prefetchQuery(projectQueryOptions.versionsV2(id, client)),
		queryClient.prefetchQuery(reviewQueryOptions.versions(id, client, queryClient)),
		queryClient.prefetchQuery(reviewQueryOptions.report(id, client)),
		disclosuresPromise.then(async (response) => {
			if (!isCurrent()) return
			const ids = [...new Set(response.disclosures.flatMap((item) => item.updated_by ?? []))]
			if (ids.length) {
				await queryClient.prefetchQuery(projectQueryOptions.disclosureUsers(ids, client))
			}
		}),
		projectPromise.then(async (project) => {
			if (!isCurrent()) return
			warmProjectCheckCaches(queryClient, project)
			await Promise.allSettled([
				project.thread_id
					? queryClient.prefetchQuery(reviewQueryOptions.thread(project.thread_id, client))
					: undefined,
				project.project_types.includes('modpack') && !project.minecraft_server
					? queryClient.prefetchQuery(projectQueryOptions.attribution(id, client))
					: undefined,
			])
		}),
		(async () => {
			const project = await projectPromise
			if (!isCurrent()) return
			const organization = project.organization
				? await queryClient
						.fetchQuery(projectQueryOptions.organization(id, client))
						.catch(() => null)
				: null
			const members = await membersPromise
			if (!isCurrent()) return
			const userIds = [
				...new Set(
					[...members, ...(organization?.members ?? [])]
						.filter((member) => member.accepted)
						.map((member) => member.user.id),
				),
			].toSorted()
			await Promise.all([
				...userIds.map((userId) =>
					queryClient.prefetchQuery(creatorProjectStatsQueryOptions.memberProjects(userId, client)),
				),
				userIds.length
					? queryClient.prefetchQuery(
							creatorProjectStatsQueryOptions.memberFlaggedProjects(userIds, client),
						)
					: undefined,
				organization
					? queryClient.prefetchQuery(
							creatorProjectStatsQueryOptions.organizationProjects(organization.id, client),
						)
					: undefined,
				organization
					? queryClient.prefetchQuery(
							creatorProjectStatsQueryOptions.organizationFlaggedProjects(organization.id, client),
						)
					: undefined,
			])
		})(),
		membersPromise,
	])
}
