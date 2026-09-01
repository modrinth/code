import { defineMessages } from '@modrinth/ui/i18n'
import LinkifyIt from 'linkify-it'
import tlds from 'tlds' with { type: 'json' }

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { validateSpam } from '../../validators/spam/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import {
	evaluateEnglishSummaryText,
	evaluateNonStandardText,
	evaluateProfanity,
	evaluateSlur,
	hasProjectTextFormatting,
	normalizeProjectFieldText,
	projectRequiresEnglishText,
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
	nonEnglish: {
		id: 'nags.project-summary-non-english.description',
		defaultMessage:
			'Your project summary must be written in English or include an English translation.',
	},
	matchesName: {
		id: 'project.text-validation.summary-matches-title',
		defaultMessage: 'A project summary cannot be the same as or too similar to its title.',
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
		defaultMessage: 'Links, URLs, or IPs are not allowed in the project summary. Detected: {value}',
	},
})

export const MIN_SUMMARY_CHARS = 25
export const MAX_SUMMARY_NAME_SIMILARITY = 0.8

export interface ProjectSummaryValidationInput {
	summary: string | null | undefined
	name: string | null | undefined
}

const summaryLinkify = new LinkifyIt({
	fuzzyEmail: false,
	fuzzyIP: true,
	fuzzyLink: true,
}).tlds(tlds)

function findProjectSummaryLinkOrIp(summary: string): string | null {
	return summaryLinkify.match(summary)?.[0].raw ?? null
}

function getLevenshteinDistance(left: string[], right: string[]) {
	if (left.length > right.length) return getLevenshteinDistance(right, left)

	let previousRow = Array.from({ length: left.length + 1 }, (_, index) => index)

	for (let rightIndex = 0; rightIndex < right.length; rightIndex++) {
		const currentRow = [rightIndex + 1]

		for (let leftIndex = 0; leftIndex < left.length; leftIndex++) {
			currentRow.push(
				Math.min(
					currentRow[leftIndex] + 1,
					previousRow[leftIndex + 1] + 1,
					previousRow[leftIndex] + (left[leftIndex] === right[rightIndex] ? 0 : 1),
				),
			)
		}

		previousRow = currentRow
	}

	return previousRow[left.length]
}

function normalizeProjectFieldTextForSimilarity(value: string) {
	return Array.from(normalizeProjectFieldText(value).toLocaleLowerCase('en-US').replace(/\s+/g, ''))
}

export function getProjectSummaryNameSimilarity(summary: string, name: string) {
	const normalizedSummary = normalizeProjectFieldTextForSimilarity(summary)
	const normalizedName = normalizeProjectFieldTextForSimilarity(name)
	const longestLength = Math.max(normalizedSummary.length, normalizedName.length)

	if (longestLength === 0) return 0

	return 1 - getLevenshteinDistance(normalizedSummary, normalizedName) / longestLength
}

export function hasProjectSummaryFormatting(summary: string) {
	return hasProjectTextFormatting(summary)
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
	'project-summary-non-english': {
		severity: 'error',
		evaluate: ({ summary }) => {
			const normalized = normalizeProjectFieldText(summary ?? '')
			if (
				!normalized ||
				normalized.length < MIN_SUMMARY_CHARS ||
				findProjectSummaryLinkOrIp(normalized) !== null ||
				!validateSpam(normalized).valid
			) {
				return { valid: true }
			}
			return evaluateEnglishSummaryText(normalized)
		},
		presentation: {
			message: messages.nonEnglish,
			nag: { title: messages.fixSummary, ...commonNagPresentation },
		},
	},
	'project-summary-matches-title': {
		severity: 'error',
		evaluate: ({ summary, name }) => ({
			valid:
				!summary ||
				findProjectSummaryLinkOrIp(summary) !== null ||
				!name ||
				getProjectSummaryNameSimilarity(summary, name) < MAX_SUMMARY_NAME_SIMILARITY,
		}),
		presentation: {
			message: messages.matchesName,
			nag: { title: messages.reviewSummary, ...commonNagPresentation },
		},
	},
	'summary-too-short': {
		severity: 'error',
		evaluate: ({ summary }) => {
			if (!summary || findProjectSummaryLinkOrIp(summary) !== null) return { valid: true }
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
		evaluate: ({ summary }) => {
			const match = summary ? findProjectSummaryLinkOrIp(summary) : null
			return match ? { valid: false, values: { value: match } } : { valid: true }
		},
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
	const matches = evaluateRules(
		{ summary: context.projectV3.summary, name: context.projectV3.name },
		projectSummaryValidationRules,
	)
	return toNags(
		projectRequiresEnglishText(context.projectV3)
			? matches
			: matches.filter(({ code }) => code !== 'project-summary-non-english'),
	)
}
