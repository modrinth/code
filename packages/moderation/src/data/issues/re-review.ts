import { RefreshCwIcon } from '@modrinth/assets'

import ignoredMessage from '../messages/checklist/messages/re-review/ignored.md'
import ignoredWarningMessage from '../messages/checklist/messages/re-review/ignored/warning.md'
import { issue, panel, toggle } from './component-builders/builders'
import type { Panel } from './component-builders/types'

export const reReviewIgnoredIssue = issue({
	id: 're-review-ignored',
	title: 'Previous review ignored',
	category: 'Project wide',
	message: ({ selected }) =>
		[
			ignoredMessage,
			selected.toggleIds.includes('re-review-warning') ? ignoredWarningMessage : '',
		].join('\n'),
	suggestedStatus: ({ selected }) =>
		selected.toggleIds.includes('re-review-warning') ? 'rejected' : 'flagged',
})

export const reReviewReviewPanel = panel({
	title: 'Re-Review',
	hint: 'Did the author ignore previous review messages?',
	icon: RefreshCwIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	shown: ({ wasReviewed, hasPreviousStructuredIssues }) =>
		wasReviewed && hasPreviousStructuredIssues,
}).content(
	toggle({
		issue: reReviewIgnoredIssue,
		label: 'Ignored issues',
	}),
	toggle({
		issue: reReviewIgnoredIssue,
		label: 'Multiple times in a row',
		id: 're-review-warning',
		issueCardDisplay: {
			label: 'Multiple times in a row',
		},
		shown: (ctx) => ctx.selected.issueIds.includes(reReviewIgnoredIssue.id),
	}),
)

export const reReviewMiscReviewPanel = {
	...reReviewReviewPanel,
	shown: ({ wasReviewed, hasPreviousStructuredIssues }) =>
		wasReviewed && !hasPreviousStructuredIssues,
} satisfies Panel
