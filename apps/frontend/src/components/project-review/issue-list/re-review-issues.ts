import { injectNotificationManager } from '@modrinth/ui'
import { type Ref, ref } from 'vue'

import type { ProjectReviewPageContext } from '~/providers/project-review'
import type { createReviewMessages } from '~/providers/project-review/review-messages'
import type { createReviewSession } from '~/providers/project-review/review-session'

export function useReReviewIssues(
	{ project, threadQuery }: Pick<ProjectReviewPageContext, 'project' | 'threadQuery'>,
	session: ReturnType<typeof createReviewSession>,
	messages: ReturnType<typeof createReviewMessages>,
	pending: Ref<boolean>,
) {
	const { handleError } = injectNotificationManager()
	const resetting = ref(false)

	async function resetIssues() {
		const current = project.value
		if (!current || pending.value || resetting.value) return
		resetting.value = true
		try {
			if (current.thread_id) {
				const result = await threadQuery.refetch({ throwOnError: true })
				if (result.data?.id !== current.thread_id) return
			}
			if (project.value?.id !== current.id || pending.value) return
			for (const scope of [
				'issue-active',
				'issues',
				'issue-text',
				'issue-select',
				'issue-order',
				'previous-issue-applicability',
				'previous-facet-applicability',
				'previous-issue-selection',
			]) {
				for (const id of Object.keys(session.read(current.id, scope))) {
					session.write(current.id, scope, id, undefined)
					if (scope !== 'previous-issue-applicability') messages.resetIssueMessage(id)
				}
			}
			for (const issue of threadQuery.data.value?.issues ?? []) {
				messages.resetIssueMessage(`previous:${issue.id}`)
			}
		} catch (error) {
			handleError(error)
		} finally {
			resetting.value = false
		}
	}

	return { resetting, resetIssues }
}
