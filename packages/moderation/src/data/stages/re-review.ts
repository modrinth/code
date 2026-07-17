import { RefreshCwIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { APPROVED_PROJECT_STATUSES, REJECTED_PROJECT_STATUSES } from '@modrinth/utils'
import { computed } from 'vue'

import { group, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project, thread } = injectProjectPageContext()

	const wasReviewed = computed(() => {
		if (project.value?.status !== 'processing') return false
		const messages = thread.value?.messages ?? []
		const lastApprovalIdx = messages.findLastIndex(
			(m) =>
				m.body.type === 'status_change' && APPROVED_PROJECT_STATUSES.includes(m.body.new_status),
		)
		return messages
			.slice(lastApprovalIdx + 1)
			.some(
				(m) =>
					m.body.type === 'status_change' && REJECTED_PROJECT_STATUSES.includes(m.body.new_status),
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
					.suggestedStatus('flagged')
					.severity('medium')
					.message()
					.children(
						toggle('warning', 'Also warn them')
							.suggestedStatus('rejected')
							.severity('high')
							.message(),
					)
					.collect(),
			),
		)
}
