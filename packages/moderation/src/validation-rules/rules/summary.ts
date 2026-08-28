import { defineMessages } from '@modrinth/ui/i18n'
import { md } from '@modrinth/utils/parse.ts'
import LinkifyIt from 'linkify-it'
import tlds from 'tlds' with { type: 'json' }

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { validateSpam } from '../../validators/spam/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import {
	evaluateNonStandardText,
	evaluateProfanity,
	evaluateSlur,
	normalizeProjectFieldText,
} from '../text.ts'
import { toFieldMessages } from '../to-field-messages.ts'
import { toNags } from '../to-nags.ts'
import type { FieldValidationMessage, ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	fixSummary: {
		id: 'nags.invalid-project-summary.title',
		defaultMessage: 'Fix the project summary',
	},
	reviewSummary: {
		id: 'nags.project-summary-content.title',
		defaultMessage: 'Review the project summary',
	},
	expandSummary: {
		id: 'nags.summary-too-short.title',
		defaultMessage: 'Expand the summary',
	},
	removeSpam: {
		id: 'nags.project-summary-spam.title',
		defaultMessage: 'Remove spam from the summary',
	},
	cleanUpSummary: {
		id: 'nags.summary-special-formatting.title',
		defaultMessage: 'Remove Markdown and HTML from the summary',
	},
	removeSummaryLinks: {
		id: 'nags.project-summary-links.title',
		defaultMessage: 'Remove links from the summary',
	},
	editSummary: {
		id: 'nags.edit-summary.title',
		defaultMessage: 'Edit summary',
	},
	slur: {
		id: 'nags.project-summary-slur.description',
		defaultMessage: 'Your project summary cannot contain any slurs. Detected: “{value}”.',
	},
	profanity: {
		id: 'nags.project-summary-profanity.description',
		defaultMessage: 'Your project summary cannot contain profanity. Detected: “{value}”.',
	},
	nonStandardText: {
		id: 'nags.project-summary-non-standard-text.description',
		defaultMessage: 'Non-standard text characters, such as “₮ɆӾ₮”, are not allowed.',
	},
	matchesName: {
		id: 'project.text-validation.summary-matches-title',
		defaultMessage: "A project summary cannot be the same as it's title.",
	},
	tooShort: {
		id: 'project.text-validation.summary-too-short',
		defaultMessage:
			'Your summary is too short. Add a sentence or two which describes your project.',
	},
	spam: {
		id: 'nags.project-summary-spam.description',
		defaultMessage:
			'Repeated characters, words, or phrases cannot be used to pad a project summary.',
	},
	specialFormatting: {
		id: 'nags.summary-special-formatting.description',
		defaultMessage: 'Your summary cannot contain Markdown or HTML, as it displays in plain text.',
	},
	links: {
		id: 'nags.project-summary-links.description',
		defaultMessage: 'Links, URLs, or IPs are not allowed in the project summary.',
	},
})

export const MIN_SUMMARY_CHARS = 25

export interface ProjectSummaryValidationInput {
	summary: string | null | undefined
	name: string | null | undefined
}

const summaryLinkify = new LinkifyIt({
	fuzzyEmail: false,
	fuzzyIP: true,
	fuzzyLink: true,
}).tlds(tlds)

const summaryMarkdown = md({ linkify: false })
const allowedSummaryBlockTokenTypes = new Set(['paragraph_open', 'inline', 'paragraph_close'])
const allowedSummaryInlineTokenTypes = new Set(['text', 'softbreak', 'hardbreak'])

function containsProjectSummaryLinkOrIp(summary: string): boolean {
	return summaryLinkify.test(summary)
}

export function projectSummaryMatchesName(summary: string, name: string) {
	const normalizedSummary = normalizeProjectFieldText(summary).replace(/\s+/g, '')
	const normalizedName = normalizeProjectFieldText(name).replace(/\s+/g, '')

	return normalizedSummary.length > 0 && normalizedSummary === normalizedName
}

export function hasProjectSummaryFormatting(summary: string) {
	return summaryMarkdown.parse(summary, {}).some((token) => {
		if (!allowedSummaryBlockTokenTypes.has(token.type)) return true

		return token.children?.some((child) => !allowedSummaryInlineTokenTypes.has(child.type)) ?? false
	})
}

const commonNagPresentation = {
	destination: 'general',
	linkTitle: messages.editSummary,
} as const

export const projectSummaryValidationRules = {
	'project-summary-slur': {
		severity: 'error',
		evaluate: ({ summary }) => evaluateSlur(summary ?? ''),
		presentation: {
			message: messages.slur,
			nag: { title: messages.fixSummary, ...commonNagPresentation },
		},
	},
	'project-summary-profanity': {
		severity: 'error',
		evaluate: ({ summary }) => evaluateProfanity(summary ?? ''),
		presentation: {
			message: messages.profanity,
			nag: { title: messages.fixSummary, ...commonNagPresentation },
		},
	},
	'project-summary-non-standard-text': {
		severity: 'error',
		evaluate: ({ summary }) => evaluateNonStandardText(summary ?? ''),
		presentation: {
			message: messages.nonStandardText,
			nag: { title: messages.fixSummary, ...commonNagPresentation },
		},
	},
	'project-summary-matches-title': {
		severity: 'error',
		evaluate: ({ summary, name }) => ({
			valid:
				!summary ||
				containsProjectSummaryLinkOrIp(summary) ||
				!name ||
				!projectSummaryMatchesName(summary, name),
		}),
		presentation: {
			message: messages.matchesName,
			nag: { title: messages.reviewSummary, ...commonNagPresentation },
		},
	},
	'summary-too-short': {
		severity: 'error',
		evaluate: ({ summary }) => {
			if (!summary || containsProjectSummaryLinkOrIp(summary)) return { valid: true }
			const length = normalizeProjectFieldText(summary).length
			return length < MIN_SUMMARY_CHARS
				? { valid: false, values: { length, minChars: MIN_SUMMARY_CHARS } }
				: { valid: true }
		},
		presentation: {
			message: messages.tooShort,
			nag: { title: messages.expandSummary, ...commonNagPresentation },
		},
	},
	'project-summary-spam': {
		severity: 'error',
		evaluate: ({ summary }) => ({
			valid: validateSpam(normalizeProjectFieldText(summary ?? '')).valid,
		}),
		presentation: {
			message: messages.spam,
			nag: { title: messages.removeSpam, ...commonNagPresentation },
		},
	},
	'summary-special-formatting': {
		severity: 'error',
		evaluate: ({ summary }) => ({
			valid: !summary || !hasProjectSummaryFormatting(summary),
		}),
		presentation: {
			message: messages.specialFormatting,
			nag: { title: messages.cleanUpSummary, ...commonNagPresentation },
		},
	},
	'project-summary-links': {
		severity: 'error',
		evaluate: ({ summary }) => ({
			valid: !summary || !containsProjectSummaryLinkOrIp(summary),
		}),
		presentation: {
			message: messages.links,
			nag: { title: messages.removeSummaryLinks, ...commonNagPresentation },
		},
	},
} satisfies ValidationRuleSet<ProjectSummaryValidationInput>

export function validateProjectSummary(
	input: ProjectSummaryValidationInput,
): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(input, projectSummaryValidationRules))
}

export function getSummaryNags(context: Pick<ProjectValidationContext, 'projectV3'>): Nag[] {
	return toNags(
		evaluateRules(
			{ summary: context.projectV3.summary, name: context.projectV3.name },
			projectSummaryValidationRules,
		),
	)
}
