import { RefreshCwIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { APPROVED_PROJECT_STATUSES, REJECTED_PROJECT_STATUSES } from '@modrinth/utils'
import { computed } from 'vue'

import { action, group, stage, toggle } from '../../types/node'

export default function () {
	const { thread } = injectProjectPageContext()

	const wasReviewed = computed(() => {
		const messages = thread.value?.messages ?? []

		const lastApprovalIdx = messages.findLastIndex(
			(m) =>
				m.body.type === 'status_change' && APPROVED_PROJECT_STATUSES.includes(m.body.new_status),
		)

		return messages
			.slice(lastApprovalIdx + 1)
			.some(
				(m) =>
					(m.body.type === 'text' && !m.body.private) ||
					(m.body.type === 'status_change' &&
						REJECTED_PROJECT_STATUSES.includes(m.body.new_status)),
			)
	})

	return stage('re-review', 'Re-Review')
		.hint('Did the author ignore previous review messages?')
		.icon(RefreshCwIcon)
		.navigate('/moderation')
		.shown(wasReviewed)
		.children(
			group().children(
				toggle('ignored', 'Yes')
					.action(action().suggestedStatus('flagged').severity('medium').message())
					.children(
						toggle('warning', 'Multiple times in a row').action(
							action().suggestedStatus('rejected').severity('high').message(),
						),
					),
			),
		)
}
