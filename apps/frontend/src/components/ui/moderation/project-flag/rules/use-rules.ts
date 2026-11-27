import type { CreateProjectFlagRuleRequest, ProjectFlagRule } from '@modrinth/api-client'
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useMutation, useMutationState, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import {
	projectFlagProjectContextQueryKey,
	projectFlagRuleQueryOptions,
} from '~/composables/queries/project-flag'

import { addUndoNotification } from './undo-notification'

const FETCH_PAGE_SIZE = 100
const UNDO_WINDOW_SECONDS = 10
const queryKey = ['moderation', 'project-flag', 'rules'] as const
const deleteRuleKey = [...queryKey, 'delete'] as const
type RuleToDelete = Pick<ProjectFlagRule, 'id' | 'name'>

export function useRules(active: () => boolean) {
	const client = injectModrinthClient()
	const api = client.labrinth.project_flags_internal
	const queryClient = useQueryClient()
	const notificationManager = injectNotificationManager()

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
	const pendingRuleIds = useMutationState({
		filters: { mutationKey: deleteRuleKey, exact: true, status: 'pending' },
		select: (mutation) => (mutation.state.variables as RuleToDelete).id,
	})
	const rules = computed(() =>
		(data.value ?? []).filter((rule) => !pendingRuleIds.value.includes(rule.id)),
	)

	const { mutate: deleteRule } = useMutation({
		mutationKey: deleteRuleKey,
		networkMode: 'always',
		retry: false,
		mutationFn: async ({ id, name }: RuleToDelete) => {
			const notificationId = `project-flag-rule-delete:${id}`
			const shouldDelete = await new Promise<boolean>((resolve) => {
				const timer = setTimeout(() => {
					notificationManager.removeNotification(notificationId)
					resolve(true)
				}, UNDO_WINDOW_SECONDS * 1000)

				addUndoNotification(notificationManager, {
					id: notificationId,
					title: 'Rule removed',
					text: `“${name}” will be permanently deleted in ${UNDO_WINDOW_SECONDS} seconds.`,
					autoCloseMs: null,
					copyable: false,
					buttons: [
						{
							label: 'Undo',
							action: () => {
								clearTimeout(timer)
								resolve(false)
							},
						},
					],
				})
			})
			if (shouldDelete) await api.delete(id)
			return shouldDelete
		},
		onSuccess: async (deleted, { id }) => {
			if (!deleted) return
			await queryClient.cancelQueries({ queryKey })
			queryClient.removeQueries({ queryKey: projectFlagRuleQueryOptions(client, id).queryKey })
			queryClient.setQueryData<ProjectFlagRule[]>(queryKey, (rules) =>
				rules?.filter((rule) => rule.id !== id),
			)
			void queryClient.invalidateQueries({ queryKey })
		},
		onError: (error) => {
			notificationManager.addNotification({
				type: 'error',
				title: 'Could not delete the rule',
				text: error.message,
			})
			void queryClient.invalidateQueries({ queryKey })
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
