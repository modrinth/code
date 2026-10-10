import { AlignLeftIcon } from '@modrinth/assets'

import formatting from '../messages/checklist/messages/summary/formatting.md'
import insufficient from '../messages/checklist/messages/summary/insufficient.md'
import nonEnglish from '../messages/checklist/messages/summary/non-english.md'
import repeatIp from '../messages/checklist/messages/summary/repeat-ip.md'
import repeatTitle from '../messages/checklist/messages/summary/repeat-title.md'
import { issue, panel, section, textarea, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

export const insufficientSummaryIssue = issue({
	id: 'summary-insufficient',
	title: 'Insufficient summary',
	category: 'Summary',
	actions: [issueTargets.modifySummary()],
	suggestedStatus: 'flagged',
	message: insufficient,
})

export const summaryRepeatsTitleIssue = issue({
	id: 'summary-repeat-title',
	title: 'Summary repeats the title',
	category: 'Summary',
	actions: [issueTargets.modifySummary()],
	suggestedStatus: 'flagged',
	message: repeatTitle,
})

export const summaryFormattingIssue = issue({
	id: 'summary-formatting',
	title: 'Invalid summary formatting',
	category: 'Summary',
	actions: [
		issueTargets.modifySummary(({ getTextValue }) => {
			const suggestion = getTextValue('summary-suggestion')
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) => {
		const suggestion = getTextValue('summary-suggestion')
		return formatting
			.replaceAll('%SUGGESTION%', () =>
				suggestion.trim()
					? `You may use the following suggested summary\n\n\`\`\`\n${suggestion}\n\`\`\``
					: '',
			)
			.trim()
	},
})

export const nonEnglishSummaryIssue = issue({
	id: 'summary-non-english',
	title: 'Non-English summary',
	category: 'Summary',
	actions: [issueTargets.modifySummary()],
	suggestedStatus: 'flagged',
	message: nonEnglish,
})

export const summaryRepeatsIpIssue = issue({
	id: 'summary-repeat-ip',
	title: 'Summary repeats the server address',
	category: 'Summary',
	actions: [issueTargets.modifySummary()],
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
				!ctx.projectV3.minecraft_java_server ||
				!!ctx.projectV3.minecraft_server?.languages?.includes('en'),
		}),
		toggle({
			label: 'Repeat of IP',
			issue: summaryRepeatsIpIssue,
			shown: (ctx) => !!ctx.projectV3.minecraft_server,
		}),
	),
	section({
		shown: ({ selected }) => selected.issueIds.includes(summaryFormattingIssue.id),
	}).content(
		textarea({
			issue: summaryFormattingIssue,
			id: 'summary-suggestion',
			label: 'Suggestion',
			maxlength: 256,
			rows: 3,
		}),
	),
)
