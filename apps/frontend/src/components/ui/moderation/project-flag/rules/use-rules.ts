import type { ProjectFlagRule, ProjectFlagRuleParams } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

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
		deleteRule,
		isDeleting,
		deleteError,
		resetDelete,
	}
}

export function useSaveRule(
	ruleId: () => string | undefined,
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
			id: string | undefined
			body: { name: string; params: ProjectFlagRuleParams }
		}) => (id === undefined ? api.create(body) : api.update(id, body)),
		onSuccess: async (rule, { id }) => {
			const created = id === undefined
			await queryClient.cancelQueries({ queryKey })
			if (created) {
				queryClient.setQueryData<ProjectFlagRule[]>(queryKey, (previous = []) => [
					rule,
					...previous.filter((entry) => entry.id !== rule.id),
				])
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
