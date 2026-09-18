import { injectModrinthClient } from '@modrinth/ui'
import { useQueries, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'

export function useReviewProject(selection: Ref<string>) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const identity = useQuery(
		computed(() => ({
			...projectQueryOptions.check(selection.value, client),
			enabled: import.meta.client && !!selection.value,
		})),
	)
	const projectId = computed(() => identity.data.value?.id ?? '')
	const projectQuery = useQuery(
		computed(() => ({
			...projectQueryOptions.v3(projectId.value, client),
			enabled: !!projectId.value,
		})),
	)
	const legacyQuery = useQuery(
		computed(() => ({
			...projectQueryOptions.v2(projectId.value, client),
			enabled: !!projectId.value,
		})),
	)
	const memberQuery = useQuery(
		computed(() => ({
			...projectQueryOptions.members(projectId.value, client),
			enabled: !!projectId.value,
		})),
	)
	const members = computed(() =>
		(memberQuery.data.value ?? [])
			.filter((member) => member.accepted)
			.toSorted((a, b) => Number(b.is_owner) - Number(a.is_owner) || a.ordering - b.ordering),
	)
	const memberProjects = useQueries({
		queries: computed(() =>
			members.value.map((member) => ({
				queryKey: ['user', member.user.id, 'projects', 'v3'],
				queryFn: () => client.labrinth.users_v3.getProjects(member.user.id),
				staleTime: 60_000,
			})),
		),
	})
	const memberStats = computed(() =>
		Object.fromEntries(
			members.value.map((member, index) => {
				const projects = memberProjects.value[index]?.data
				return [
					member.user.id,
					projects
						? Object.entries(Object.groupBy(projects, (project) => project.status)).map(
								([status, projects]) => ({
									status,
									count: projects?.length ?? 0,
								}),
							)
						: [],
				]
			}),
		),
	)
	const threadId = computed(() => projectQuery.data.value?.thread_id ?? '')
	const threadQuery = useQuery({
		queryKey: computed(() => ['thread', threadId.value]),
		queryFn: () => client.labrinth.threads_v3.getThread(threadId.value),
		enabled: computed(() => !!threadId.value),
	})
	const submissionCount = computed(
		() =>
			threadQuery.data.value?.messages.filter(
				(message) =>
					message.body.type === 'status_change' && message.body.new_status === 'processing',
			).length,
	)

	async function refresh() {
		await Promise.all([
			queryClient.invalidateQueries({
				queryKey: ['project', 'v3', projectId.value],
			}),
			queryClient.invalidateQueries({
				queryKey: ['project', 'v2', projectId.value],
			}),
			queryClient.invalidateQueries({ queryKey: ['project', projectId.value] }),
			identity.refetch(),
		])
	}

	return {
		projectId,
		gallery: computed(() =>
			(projectQuery.data.value?.gallery ?? []).toSorted((a, b) => a.ordering - b.ordering),
		),
		project: projectQuery.data,
		projectV2: legacyQuery.data,
		threadQuery,
		members,
		memberStats,
		membersLoading: memberQuery.isPending,
		membersError: memberQuery.isError,
		compatibilityError: legacyQuery.isError,
		submissionCount,
		isLoading: computed(
			() =>
				!!selection.value &&
				(identity.isPending.value || projectQuery.isPending.value) &&
				!identity.isError.value,
		),
		error: computed(() => identity.error.value ?? projectQuery.error.value),
		refresh,
	}
}
