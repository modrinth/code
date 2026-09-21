import { AlignLeftIcon } from '@modrinth/assets'

import formatting from '../messages/checklist/messages/summary/formatting.md'
import insufficient from '../messages/checklist/messages/summary/insufficient.md'
import nonEnglish from '../messages/checklist/messages/summary/non-english.md'
import repeatIp from '../messages/checklist/messages/summary/repeat-ip.md'
import repeatTitle from '../messages/checklist/messages/summary/repeat-title.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const insufficientSummaryIssue = issue({
	id: 'summary-insufficient',
	suggestedStatus: 'flagged',
	message: insufficient,
})

export const summaryRepeatsTitleIssue = issue({
	id: 'summary-repeat-title',
	suggestedStatus: 'flagged',
	message: repeatTitle,
})

export const summaryFormattingIssue = issue({
	id: 'summary-formatting',
	suggestedStatus: 'flagged',
	message: formatting,
})

export const nonEnglishSummaryIssue = issue({
	id: 'summary-non-english',
	suggestedStatus: 'flagged',
	message: nonEnglish,
})

export const summaryRepeatsIpIssue = issue({
	id: 'summary-repeat-ip',
	suggestedStatus: 'flagged',
	message: repeatIp,
})

export const summaryReviewPanel = panel({
	field: 'summary',
	title: 'Summary',
	hint: "Is the project's summary sufficient?",
	icon: AlignLeftIcon,
}).content(
	section().content(
		toggle({
			label: 'Insufficient',
			issue: insufficientSummaryIssue,
		}),
		toggle({
			label: 'Repeat of Title',
			issue: summaryRepeatsTitleIssue,
		}),
		toggle({
			label: 'Formatting',
			issue: summaryFormattingIssue,
		}),
		toggle({
			label: 'Non-english',
			issue: nonEnglishSummaryIssue,
			shown: (ctx) =>
				!ctx.ProjectV3.minecraft_java_server ||
				!!ctx.ProjectV3.minecraft_server?.languages?.includes('en'),
		}),
		toggle({
			label: 'Repeat of IP',
			issue: summaryRepeatsIpIssue,
			shown: (ctx) => !!ctx.ProjectV3.minecraft_server,
		}),
	),
)
