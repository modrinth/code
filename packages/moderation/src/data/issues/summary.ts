import { AlignLeftIcon } from '@modrinth/assets'

import formatting from '../messages/checklist/messages/summary/formatting.md'
import insufficient from '../messages/checklist/messages/summary/insufficient.md'
import nonEnglish from '../messages/checklist/messages/summary/non-english.md'
import repeatIp from '../messages/checklist/messages/summary/repeat-ip.md'
import repeatTitle from '../messages/checklist/messages/summary/repeat-title.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const insufficientSummaryIssue = issue({
	id: 'summary-insufficient',
	title: 'Insufficient summary',
	category: 'Summary',
	suggestedStatus: 'flagged',
	message: insufficient,
})

export const summaryRepeatsTitleIssue = issue({
	id: 'summary-repeat-title',
	title: 'Summary repeats the title',
	category: 'Summary',
	suggestedStatus: 'flagged',
	message: repeatTitle,
})

export const summaryFormattingIssue = issue({
	id: 'summary-formatting',
	title: 'Invalid summary formatting',
	category: 'Summary',
	suggestedStatus: 'flagged',
	message: formatting,
})

export const nonEnglishSummaryIssue = issue({
	id: 'summary-non-english',
	title: 'Non-English summary',
	category: 'Summary',
	suggestedStatus: 'flagged',
	message: nonEnglish,
})

export const summaryRepeatsIpIssue = issue({
	id: 'summary-repeat-ip',
	title: 'Summary repeats the server address',
	category: 'Summary',
	suggestedStatus: 'flagged',
	message: repeatIp,
})

export const summaryReviewPanel = panel({
	title: 'Summary',
	hint: "Is the project's summary sufficient?",
	icon: AlignLeftIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
}).content(
	section().content(
		toggle({
			label: 'Insufficient',
			issue: insufficientSummaryIssue,
			disabled: ({ selected }) => selected.issueIds.includes(summaryRepeatsTitleIssue.id),
		}),
		toggle({
			label: 'Repeat of Title',
			issue: summaryRepeatsTitleIssue,
			disabled: ({ selected }) => selected.issueIds.includes(insufficientSummaryIssue.id),
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
