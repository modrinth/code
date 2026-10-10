import { AlignLeftIcon } from '@modrinth/assets'

import formatting from '../messages/checklist/messages/summary/formatting.md'
import insufficient from '../messages/checklist/messages/summary/insufficient.md'
import nonEnglish from '../messages/checklist/messages/summary/non-english.md'
import repeatIp from '../messages/checklist/messages/summary/repeat-ip.md'
import { issue, panel, section, textarea, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'
import { messageWithOptionalSuggestion } from '../../utils'

const panelCategory = 'Summary'

export const insufficientSummaryIssue = issue({
	id: 'summary-insufficient',
	title: 'Insufficient summary',
	category: panelCategory,
	actions: [
		issueTargets.modifySummary(({ getTextValue }) => {
			const suggestion = getTextValue('summary-suggestion')
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(
			insufficient,
			panelCategory.toLowerCase(),
			getTextValue('summary-suggestion'),
		),
})

export const summaryFormattingIssue = issue({
	id: 'summary-formatting',
	title: 'Invalid summary formatting',
	category: panelCategory,
	actions: [
		issueTargets.modifySummary(({ getTextValue }) => {
			const suggestion = getTextValue('summary-suggestion')
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(
			formatting,
			panelCategory.toLowerCase(),
			getTextValue('summary-suggestion'),
		),
})

export const nonEnglishSummaryIssue = issue({
	id: 'summary-non-english',
	title: 'Non-English summary',
	category: panelCategory,
	actions: [
		issueTargets.modifySummary(({ getTextValue }) => {
			const suggestion = getTextValue('summary-suggestion')
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(
			nonEnglish,
			panelCategory.toLowerCase(),
			getTextValue('summary-suggestion'),
		),
})

export const summaryRepeatsIpIssue = issue({
	id: 'summary-repeat-ip',
	title: 'Summary repeats the server address',
	category: panelCategory,
	actions: [
		issueTargets.modifySummary(({ getTextValue }) => {
			const suggestion = getTextValue('summary-suggestion')
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	suggestedStatus: 'flagged',
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(
			repeatIp,
			panelCategory.toLowerCase(),
			getTextValue('summary-suggestion'),
		),
})

export const summaryReviewPanel = panel({
	title: panelCategory,
	hint: "Is the project's summary sufficient?",
	icon: AlignLeftIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
}).content(
	section().content(
		toggle({
			label: 'Insufficient',
			issue: insufficientSummaryIssue,
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
		shown: ({ selected }) =>
			selected.issueIds.some((id) =>
				[
					insufficientSummaryIssue.id,
					summaryFormattingIssue.id,
					nonEnglishSummaryIssue.id,
					summaryRepeatsIpIssue.id,
				].includes(id),
			),
	}).content(
		textarea({
			id: 'summary-suggestion',
			label: 'Suggestion',
			maxlength: 256,
			rows: 3,
		}),
	),
)
