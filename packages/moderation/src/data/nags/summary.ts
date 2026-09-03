import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	linksTitle: {
		id: 'nags.project-summary-links.title',
		defaultMessage: 'Remove summary links',
	},
	links: {
		id: 'nags.project-summary-links.description',
		defaultMessage: 'Links, URLs, and IPs should not be placed in the summary. Detected: "{value}"',
	},
	reviewTitle: {
		id: 'nags.project-summary-content.title',
		defaultMessage: 'Review the summary',
	},
	matchesTitle: {
		id: 'project.text-validation.summary-matches-title',
		defaultMessage: `Your project's summary should provide unique information and not repeat the project's name.`,
	},
	fixTitle: { id: 'nags.invalid-project-summary.title', defaultMessage: 'Modify the summary' },
	nonEnglish: {
		id: 'nags.project-summary-non-english.description',
		defaultMessage: `Your project's summary must be written in English or include an English translation.`,
	},
	nonStandardText: {
		id: 'nags.project-summary-non-standard-text.description',
		defaultMessage:
			'Non-standard text characters, such as “Fancy text” or “Zalgo”, are not allowed in the summary.',
	},
	profanity: {
		id: 'nags.project-summary-profanity.description',
		defaultMessage: `Your project's summary cannot contain profanity. Detected: “{value}”.`,
	},
	slur: {
		id: 'nags.project-summary-slur.description',
		defaultMessage: `Your project's summary must not contain offensive terms. Detected: “{value}”.`,
	},
	spamTitle: {
		id: 'nags.project-summary-spam.title',
		defaultMessage: 'Remove summary spam',
	},
	spam: {
		id: 'nags.project-summary-spam.description',
		defaultMessage: `Repeated characters, words, or phrases should not be used to pad your project's summary.`,
	},
	formattingTitle: {
		id: 'nags.summary-special-formatting.title',
		defaultMessage: 'Fix summary formatting',
	},
	formatting: {
		id: 'nags.summary-special-formatting.description',
		defaultMessage:
			'Your summary should not contain Markdown or HTML, since it can only display plain text.',
	},
	shortTitle: { id: 'nags.summary-too-short.title', defaultMessage: 'Expand the summary' },
	short: {
		id: 'project.text-validation.summary-too-short',
		defaultMessage: 'Your summary is too brief. Add a sentence or two that describes your project.',
	},
	editSummary: { id: 'nags.edit-summary.title', defaultMessage: 'Edit summary' },
})

export const summaryNags = {
	'project-summary-links': {
		title: messages.linksTitle,
		description: messages.links,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-matches-title': {
		title: messages.reviewTitle,
		description: messages.matchesTitle,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-non-english': {
		title: messages.fixTitle,
		description: messages.nonEnglish,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-non-standard-text': {
		title: messages.fixTitle,
		description: messages.nonStandardText,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-profanity': {
		title: messages.fixTitle,
		description: messages.profanity,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-slur': {
		title: messages.fixTitle,
		description: messages.slur,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'project-summary-spam': {
		title: messages.spamTitle,
		description: messages.spam,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'summary-special-formatting': {
		title: messages.formattingTitle,
		description: messages.formatting,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
	'summary-too-short': {
		title: messages.shortTitle,
		description: messages.short,
		destination: 'general',
		linkTitle: messages.editSummary,
	},
} satisfies NagDefinitions
