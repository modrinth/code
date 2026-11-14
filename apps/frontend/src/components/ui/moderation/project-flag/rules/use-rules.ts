import type { ProjectFlagRule, ProjectFlagRuleParams } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

export const RULES_PAGE_SIZE = 25
const queryKey = ['moderation', 'project-flag', 'rules'] as const

function rulesPageKey(pageOffset: number) {
	return [...queryKey, { limit: RULES_PAGE_SIZE + 1, offset: pageOffset }] as const
}

export function useRules(offset: Ref<number>, active: () => boolean, onDeleted: () => void) {
	const client = injectModrinthClient()
	const api = client.labrinth.project_flags_internal
	const queryClient = useQueryClient()

	const { data, isPending, isError, error, isFetching, refetch } = useQuery({
		queryKey: computed(() => rulesPageKey(offset.value)),
		queryFn: ({ queryKey: [, , , page] }) => api.list(page),
		enabled: computed(() => active()),
	})
	const rules = computed(() => data.value?.slice(0, RULES_PAGE_SIZE) ?? [])
	const hasNextPage = computed(() => (data.value?.length ?? 0) > RULES_PAGE_SIZE)

	const {
		mutate: deleteRule,
		isPending: isDeleting,
		error: deleteError,
		reset: resetDelete,
	} = useMutation({
		mutationFn: (id: number) => api.delete(id),
		onSuccess: async () => {
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
		hasNextPage,
		deleteRule,
		isDeleting,
		deleteError,
		resetDelete,
	}
}

export function useSaveRule(
	ruleId: () => number | undefined,
	onSaved: (rule: ProjectFlagRule) => void,
) {
	const { labrinth } = injectModrinthClient()
	const api = labrinth.project_flags_internal
	const queryClient = useQueryClient()
	const mutation = useMutation({
		mutationFn: ({
			id,
			body,
		}: {
			id: number | undefined
			body: { name: string; params: ProjectFlagRuleParams }
		}) => (id === undefined ? api.create(body) : api.update(id, body)),
		onSuccess: async (rule, { id }) => {
			const created = id === undefined
			await queryClient.cancelQueries({ queryKey })
			if (created) {
				queryClient.setQueryData<ProjectFlagRule[]>(rulesPageKey(0), (previous = []) =>
					[rule, ...previous.filter((entry) => entry.id !== rule.id)].slice(0, RULES_PAGE_SIZE + 1),
				)
			} else {
				queryClient.setQueriesData<ProjectFlagRule[]>({ queryKey }, (previous) =>
					previous?.map((entry) => (entry.id === rule.id ? rule : entry)),
				)
			}
			onSaved(rule)
			await queryClient.invalidateQueries({ queryKey })
		},
	})
	return {
		...mutation,
		mutate: (body: { name: string; params: ProjectFlagRuleParams }) =>
			mutation.mutate({ id: ruleId(), body }),
	}
}
