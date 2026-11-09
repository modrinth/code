import type { AbstractModrinthClient } from '@modrinth/api-client'
import { getRemovedScopes, injectModrinthClient, useVIntl } from '@modrinth/ui'
import { useQueries, useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

export const creatorProjectStatsQueryOptions = {
	memberAccount: (id: string, client: AbstractModrinthClient) => ({
		queryKey: ['user', id, 'moderation-account'] as const,
		queryFn: () => client.labrinth.users_v3.get(id),
		staleTime: 60_000,
	}),
	memberBlacklist: (id: string, client: AbstractModrinthClient) => ({
		queryKey: ['shared-instance-blacklist', 'v1', id] as const,
		queryFn: () => client.sharedinstances.users_v1.getBlacklistStatus(id),
		staleTime: 60_000,
	}),
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
	const { formatMessage } = useVIntl()
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
	const memberAccounts = useQueries({
		queries: computed(() =>
			memberIds.value.map((id) => ({
				...creatorProjectStatsQueryOptions.memberAccount(id, client),
				enabled: enabled.value,
			})),
		),
	})
	const memberBlacklists = useQueries({
		queries: computed(() =>
			memberIds.value.map((id) => ({
				...creatorProjectStatsQueryOptions.memberBlacklist(id, client),
				enabled: enabled.value,
			})),
		),
	})
	const memberStats = computed(() =>
		Object.fromEntries(
			memberIds.value.map((userId, index) => {
				const projects = memberProjects.value[index]?.data
				const account = memberAccounts.value[index]?.data
				const restriction = account?.restriction
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
						{
							status: 'shared_instances_banned',
							count: memberBlacklists.value[index]?.data?.blacklisted ? 1 : 0,
						},
						{
							status: 'account_locked',
							count: account?.lock ? 1 : 0,
							details: account?.lock?.reason,
						},
						{
							status: 'account_restricted',
							count: restriction?.removed_perms ? 1 : 0,
							details: restriction
								? [
										...getRemovedScopes(restriction.removed_perms).map((scope) =>
											formatMessage(scope.label),
										),
										restriction.reason,
										restriction.private_reason,
									].filter((detail): detail is string => !!detail)
								: undefined,
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
