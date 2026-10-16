import type { AbstractModrinthClient } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useQueries, useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

export const creatorProjectStatsQueryOptions = {
	organizationProjects: (id: string, client: AbstractModrinthClient) => ({
		queryKey: ['organization', id, 'projects'] as const,
		queryFn: () => client.labrinth.organizations_v3.getProjects(id),
		staleTime: 60_000,
	}),
	organizationFlaggedProjects: (id: string, client: AbstractModrinthClient) => ({
		queryKey: ['tech-reviews', 'flagged-projects', 'organization', id] as const,
		queryFn: () => client.labrinth.tech_review_internal.getOrganizationFlaggedProjects(id),
		staleTime: 60_000,
	}),
	memberFlaggedProjects: (ids: string[], client: AbstractModrinthClient) => ({
		queryKey: ['tech-reviews', 'flagged-projects', 'users', ids] as const,
		queryFn: () => client.labrinth.tech_review_internal.getUsersFlaggedProjects(ids),
		staleTime: 60_000,
	}),
	memberProjects: (id: string, client: AbstractModrinthClient) => ({
		queryKey: ['user', id, 'projects', 'v3'] as const,
		queryFn: () => client.labrinth.users_v3.getProjects(id),
		staleTime: 60_000,
	}),
}

export function useCreatorProjectStats(
	userIds: Ref<string[]>,
	organizationId: Ref<string>,
	enabled: Ref<boolean> = computed(() => true),
) {
	const client = injectModrinthClient()
	const memberIds = computed(() => (enabled.value ? [...new Set(userIds.value)].toSorted() : []))
	const organizationProjects = useQuery(
		computed(() => ({
			...creatorProjectStatsQueryOptions.organizationProjects(organizationId.value, client),
			enabled: enabled.value && !!organizationId.value,
		})),
	)
	const organizationFlaggedProjects = useQuery(
		computed(() => ({
			...creatorProjectStatsQueryOptions.organizationFlaggedProjects(organizationId.value, client),
			enabled: enabled.value && !!organizationId.value,
		})),
	)
	const organizationStats = computed(() => [
		...Object.entries(
			Object.groupBy(organizationProjects.data.value ?? [], (project) => project.status),
		).map(([status, projects]) => ({ status, count: projects?.length ?? 0 })),
		{
			status: 'tech_review_failed',
			count: organizationFlaggedProjects.data.value?.length ?? 0,
		},
	])
	const memberFlaggedProjects = useQuery(
		computed(() => ({
			...creatorProjectStatsQueryOptions.memberFlaggedProjects(memberIds.value, client),
			enabled: enabled.value && memberIds.value.length > 0,
		})),
	)
	const memberProjects = useQueries({
		queries: computed(() =>
			memberIds.value.map((userId) => ({
				...creatorProjectStatsQueryOptions.memberProjects(userId, client),
				enabled: enabled.value,
			})),
		),
	})
	const memberStats = computed(() =>
		Object.fromEntries(
			memberIds.value.map((userId, index) => {
				const projects = memberProjects.value[index]?.data
				return [
					userId,
					[
						...Object.entries(Object.groupBy(projects ?? [], (project) => project.status)).map(
							([status, projects]) => ({
								status,
								count: projects?.length ?? 0,
							}),
						),
						{
							status: 'tech_review_failed',
							count: memberFlaggedProjects.data.value?.[userId]?.length ?? 0,
						},
					],
				]
			}),
		),
	)
	return {
		memberStats,
		organizationStats,
		membersLoading: computed(
			() =>
				memberFlaggedProjects.isLoading.value ||
				memberProjects.value.some((query) => query.isLoading),
		),
		membersError: computed(
			() =>
				memberFlaggedProjects.isError.value || memberProjects.value.some((query) => query.isError),
		),
		organizationLoading: computed(
			() => organizationProjects.isLoading.value || organizationFlaggedProjects.isLoading.value,
		),
		organizationError: computed(
			() => organizationProjects.isError.value || organizationFlaggedProjects.isError.value,
		),
	}
}
