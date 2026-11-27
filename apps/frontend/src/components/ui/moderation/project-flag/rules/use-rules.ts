import type { CreateProjectFlagRuleRequest, ProjectFlagRule } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import {
	projectFlagProjectContextQueryKey,
	projectFlagRuleQueryOptions,
} from '~/composables/queries/project-flag'

const FETCH_PAGE_SIZE = 100
const queryKey = ['moderation', 'project-flag', 'rules'] as const

export function useRules(active: () => boolean, onDeleted: () => void) {
	const client = injectModrinthClient()
	const api = client.labrinth.project_flags_internal
	const queryClient = useQueryClient()

	const { data, isPending, isError, error, isFetching, refetch } = useQuery({
		queryKey,
		queryFn: async ({ signal }) => {
			const rules: ProjectFlagRule[] = []
			for (let offset = 0; ; offset += FETCH_PAGE_SIZE) {
				signal.throwIfAborted()
				const page = await api.list({ limit: FETCH_PAGE_SIZE, offset })
				rules.push(...page)
				if (page.length < FETCH_PAGE_SIZE) return rules
			}
		},
		enabled: computed(() => active()),
	})
	const rules = computed(() => data.value ?? [])

	const {
		mutate: deleteRule,
		isPending: isDeleting,
		error: deleteError,
		reset: resetDelete,
	} = useMutation({
		mutationFn: (id: string) => api.delete(id),
		onSuccess: async (_, id) => {
			queryClient.removeQueries({ queryKey: projectFlagRuleQueryOptions(client, id).queryKey })
			onDeleted()
			await queryClient.invalidateQueries({ queryKey })
		},
	})

	return {
		data,
		isPending,
		isError,
		error,
		isFetching,
		refetch,
		rules,
		deleteRule,
		isDeleting,
		deleteError,
		resetDelete,
	}
}

export function useSaveRule() {
	const client = injectModrinthClient()
	const api = client.labrinth.project_flags_internal
	const queryClient = useQueryClient()
	return useMutation({
		mutationFn: ({ id, body }: { id: string | undefined; body: CreateProjectFlagRuleRequest }) =>
			id === undefined ? api.create(body) : api.update(id, body),
		onSuccess: async (rule, { id }) => {
			await queryClient.cancelQueries({ queryKey })
			queryClient.setQueryData<ProjectFlagRule[]>(queryKey, (previous) =>
				id === undefined
					? previous && [rule, ...previous.filter((entry) => entry.id !== rule.id)]
					: previous?.map((entry) => (entry.id === rule.id ? rule : entry)),
			)
			queryClient.setQueryData(projectFlagRuleQueryOptions(client, rule.id).queryKey, rule)
			await Promise.all([
				queryClient.invalidateQueries({ queryKey }),
				queryClient.resetQueries({ queryKey: projectFlagProjectContextQueryKey }),
			])
		},
	})
}
