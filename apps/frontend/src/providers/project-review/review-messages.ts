import type { Labrinth } from '@modrinth/api-client'
import { expandVariables } from '@modrinth/moderation/src/utils'
import { createContext } from '@modrinth/ui'
import { computed, reactive, type Ref, watch } from 'vue'

import type { createReviewPanels } from './review-panels'

export const [injectReviewMessages, provideReviewMessages] =
	createContext<ReturnType<typeof createReviewMessages>>('ProjectReviewMessages')

export function createReviewMessages(
	project: Ref<Labrinth.Projects.v3.Project | undefined>,
	projectV2: Ref<Labrinth.Projects.v2.Project | undefined>,
	panels: ReturnType<typeof createReviewPanels>,
) {
	const overrides = reactive(new Map<string, string>())
	const defaults = reactive(new Map<string, string>())
	const generating = computed(() => !!project.value && projectV2.value?.id !== project.value.id)
	const issueMessages = computed(() => {
		const current = project.value
		const legacy = projectV2.value
		if (!current || !legacy || current.id !== legacy.id) return new Map<string, string>()
		return new Map(
			panels.activeIssues.value.map(({ id, issue }) => [
				id,
				expandVariables(issue.message.trim(), legacy, current),
			]),
		)
	})
	function issueMessage(id: string) {
		const custom = panels.customIssues.value.find((issue) => issue.id === id)?.custom
		if (custom) return custom.message
		return overrides.get(id) ?? issueMessages.value.get(id) ?? defaults.get(id) ?? ''
	}
	function editIssueMessage(id: string, message: string) {
		if (panels.customIssues.value.some((issue) => issue.id === id)) {
			panels.updateCustomIssue(id, { message })
			return
		}
		if (message === issueMessage(id)) return
		const defaultMessage = issueMessages.value.get(id) ?? defaults.get(id)
		if (message === defaultMessage) overrides.delete(id)
		else overrides.set(id, message)
	}
	function resetIssueMessage(id: string) {
		overrides.delete(id)
		panels.updateCustomIssue(id, { message: '' })
	}
	watch(
		() => project.value?.id,
		() => {
			overrides.clear()
			defaults.clear()
		},
		{ flush: 'sync' },
	)

	return {
		generating,
		issueMessage,
		setIssueDefault: (id: string, message: string) => {
			defaults.set(id, message)
		},
		editIssueMessage,
		resetIssueMessage,
		hasIssueOverride: (id: string) => overrides.has(id),
	}
}
