import { RefreshCwIcon } from '@modrinth/assets'

import ignoredMessage from '../messages/checklist/messages/re-review/ignored.md'
import ignoredWarningMessage from '../messages/checklist/messages/re-review/ignored/warning.md'
import { issue, panel, section, toggle } from './component-builders/builders'

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
	shown: ({ wasReviewed }) => wasReviewed,
}).content(
	toggle({
		issue: reReviewIgnoredIssue,
		label: 'Yes',
	}),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(reReviewIgnoredIssue.id),
	}).content(
		toggle({
			issue: reReviewIgnoredIssue,
			label: 'Multiple times in a row',
			id: 're-review-warning',
		}),
	),
)
