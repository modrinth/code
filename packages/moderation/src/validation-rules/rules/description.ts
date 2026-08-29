import { defineMessages } from '@modrinth/ui/i18n'
import { renderString } from '@modrinth/utils/parse.ts'
import LinkifyIt from 'linkify-it'
import { parse } from 'node-html-parser'
import tlds from 'tlds' with { type: 'json' }

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { URL_SHORTENERS } from '../../validators/links/block-list.ts'
import {
	getLinkHostname,
	hostnameMatchesDomain,
	isIpAddress,
} from '../../validators/links/syntax-checks.ts'
import { validateSpam } from '../../validators/spam/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import {
	evaluateEnglishTextBlocks,
	evaluateNonStandardText,
	evaluateProfanity,
	evaluateSlur,
	normalizeProjectFieldText,
	projectRequiresEnglishText,
} from '../text.ts'
import { toFieldMessages } from '../to-field-messages.ts'
import { toNags } from '../to-nags.ts'
import type { FieldValidationMessage, ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	fixDescription: {
		id: 'nags.invalid-project-description.title',
		defaultMessage: 'Fix the project description',
	},
	addDescription: {
		id: 'nags.add-description.title',
		defaultMessage: 'Add a description',
	},
	expandDescription: {
		id: 'nags.description-too-short.title',
		defaultMessage: 'Expand the description',
	},
	removeSpam: {
		id: 'nags.project-description-spam.title',
		defaultMessage: 'Remove spam from the description',
	},
	shortenHeaders: {
		id: 'nags.long-headers.title',
		defaultMessage: 'Shorten headers',
	},
	addImageAltText: {
		id: 'nags.missing-alt-text.title',
		defaultMessage: 'Add image alt text',
	},
	editDescription: {
		id: 'nags.edit-description.title',
		defaultMessage: 'Edit description',
	},
	slur: {
		id: 'nags.project-description-slur.description',
		defaultMessage: 'Your project cannot contain any slurs. Detected: “{value}”.',
	},
	profanity: {
		id: 'nags.project-description-profanity.description',
		defaultMessage: 'Your project cannot contain excessive profanity. Detected: “{value}”.',
	},
	nonStandardText: {
		id: 'nags.project-description-non-standard-text.description',
		defaultMessage: 'Non-standard text characters, such as “₮ɆӾ₮”, are not allowed.',
	},
	nonEnglish: {
		id: 'nags.project-description-non-english.description',
		defaultMessage:
			'Your project description must be written in English or include an English translation.',
	},
	bannedLink: {
		id: 'nags.project-description-banned-link.description',
		defaultMessage: '“{fullUrl}” is not allowed in project descriptions.',
	},
	required: {
		id: 'nags.add-description.description',
		defaultMessage:
			"A description that clearly describes the project's purpose and function is required.",
	},
	tooShort: {
		id: 'nags.description-too-short.description',
		defaultMessage:
			"Your description is too brief. Add more to clearly describe the project's purpose and function.",
	},
	spam: {
		id: 'nags.project-description-spam.description',
		defaultMessage:
			'Repeated characters, words, or phrases cannot be used to pad a project description.',
	},
	longHeaders: {
		id: 'nags.long-headers.description',
		defaultMessage:
			'{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
	},
	missingAltText: {
		id: 'nags.missing-alt-text.description',
		defaultMessage:
			'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
	},
})

export const DESCRIPTION_MAX_PROFANITY_COUNT = 2
export const DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD = 0.05
export const MIN_DESCRIPTION_CHARS = 125
export const MAX_HEADER_LENGTH = 80
export const BANNED_DESCRIPTION_LINK_DOMAINS = [...URL_SHORTENERS] as const

const descriptionLinkify = new LinkifyIt({
	fuzzyEmail: false,
	fuzzyIP: true,
	fuzzyLink: true,
}).tlds(tlds)

const headerCharacterSegmenter = new Intl.Segmenter(undefined, { granularity: 'grapheme' })

export function extractDescriptionLinks(description: string): string[] {
	const matches = descriptionLinkify.match(description) ?? []
	return [
		...new Set(
			matches
				.map((match) => match.url)
				.filter((url) => {
					const hostname = getLinkHostname(url)
					return hostname === null || !isIpAddress(hostname)
				}),
		),
	]
}

export function findBannedDescriptionLink(description: string): string | null {
	for (const url of extractDescriptionLinks(description)) {
		const hostname = getLinkHostname(url)
		if (
			hostname &&
			BANNED_DESCRIPTION_LINK_DOMAINS.some((domain) => hostnameMatchesDomain(hostname, domain))
		) {
			return url
		}
	}

	return null
}

export function extractRenderedHeaders(markdown: string): string[] {
	if (!markdown) return []

	const renderedDescription = parse(renderString(markdown))
	return renderedDescription
		.querySelectorAll('h1, h2, h3')
		.map((header) => header.textContent.replace(/\s+/g, ' ').trim())
}

function countHeaderCharacters(header: string): number {
	return [...headerCharacterSegmenter.segment(header)].length
}

export function analyzeHeaderLength(markdown: string): {
	hasLongHeaders: boolean
	longHeaders: string[]
} {
	const longHeaders = extractRenderedHeaders(markdown).filter(
		(header) => countHeaderCharacters(header) > MAX_HEADER_LENGTH,
	)

	return { hasLongHeaders: longHeaders.length > 0, longHeaders }
}

export function extractDescriptionText(markdown: string): string {
	if (!markdown) return ''

	const withoutCode = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const withoutImagesAndLinks = withoutCode
		.replace(/!\[([^\]]*)]\([^)]+\)/g, '$1')
		.replace(/\[[^\]]*]\([^)]+\)/g, ' ')
	const withHtmlImageAltText = withoutImagesAndLinks.replace(/<img[^>]*>/gi, (image) => {
		const altMatch = image.match(/alt\s*=\s*(?:"([^"]*)"|'([^']*)')/i)
		return altMatch?.[1] ?? altMatch?.[2] ?? ' '
	})
	const withoutHtml = withHtmlImageAltText.replace(/<[^>]+>/g, ' ')
	const withoutMarkdownSyntax = withoutHtml
		.replace(/^(?:>[ \t]?)+/gm, '')
		.replace(/^#{1,6}\s+/gm, ' ')
		.replace(/[*_~`>-]/g, ' ')
		.replace(/\|/g, ' ')

	return withoutMarkdownSyntax.replace(/\s+/g, ' ').trim()
}

export function extractDescriptionTextBlocks(markdown: string): string[] {
	if (!markdown) return []

	return markdown
		.replace(/```[\s\S]*?```/g, '')
		.split(/\n\s*\n+/)
		.map(extractDescriptionText)
		.filter(Boolean)
}

export function countText(markdown: string): number {
	return extractDescriptionText(markdown).length
}

export function analyzeImageContent(markdown: string): {
	hasEmptyAltText: boolean
} {
	if (!markdown) return { hasEmptyAltText: false }

	const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const images = [...withoutCodeBlocks.matchAll(/!\[([^\]]*)\]\([^)]+\)/g)]
	const htmlImages = [...withoutCodeBlocks.matchAll(/<img[^>]*>/gi)]
	const hasEmptyAltText =
		images.some((match) => !match[1]?.trim()) ||
		htmlImages.some((match) => {
			const altMatch = match[0].match(/alt\s*=\s*["']([^"']*)["']/i)
			return !altMatch || !altMatch[1]?.trim()
		})

	return { hasEmptyAltText }
}

type DescriptionInput = string | null | undefined

const commonNagPresentation = {
	destination: 'description',
	linkTitle: messages.editDescription,
} as const

export const projectDescriptionValidationRules = {
	'project-description-slur': {
		severity: 'error',
		evaluate: (description) => evaluateSlur(description ?? ''),
		presentation: {
			message: messages.slur,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'project-description-profanity': {
		severity: 'error',
		evaluate: (description) =>
			evaluateProfanity(description ?? '', DESCRIPTION_MAX_PROFANITY_COUNT),
		presentation: {
			message: messages.profanity,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'project-description-non-standard-text': {
		severity: 'error',
		evaluate: (description) =>
			evaluateNonStandardText(description ?? '', DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD),
		presentation: {
			message: messages.nonStandardText,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'project-description-non-english': {
		severity: 'warning',
		evaluate: (description) => {
			const blocks = extractDescriptionTextBlocks(description ?? '')
			const text = blocks.join(' ')
			if (text.length < MIN_DESCRIPTION_CHARS || !validateSpam(text).valid) {
				return { valid: true }
			}

			return evaluateEnglishTextBlocks(blocks)
		},
		presentation: {
			message: messages.nonEnglish,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'add-description': {
		severity: 'error',
		evaluate: (description) => ({
			valid: normalizeProjectFieldText(description ?? '').length > 0,
		}),
		presentation: {
			message: messages.required,
			nag: { title: messages.addDescription, ...commonNagPresentation },
		},
	},
	'description-too-short': {
		severity: 'error',
		evaluate: (description) => {
			const normalized = normalizeProjectFieldText(description ?? '')
			if (!normalized) return { valid: true }

			const length = countText(normalized)
			return length >= MIN_DESCRIPTION_CHARS
				? { valid: true }
				: { valid: false, values: { length, minChars: MIN_DESCRIPTION_CHARS } }
		},
		presentation: {
			message: messages.tooShort,
			nag: { title: messages.expandDescription, ...commonNagPresentation },
		},
	},
	'project-description-spam': {
		severity: 'error',
		evaluate: (description) => ({
			valid: validateSpam(extractDescriptionText(description ?? '')).valid,
		}),
		presentation: {
			message: messages.spam,
			nag: { title: messages.removeSpam, ...commonNagPresentation },
		},
	},
	'project-description-banned-link': {
		severity: 'error',
		evaluate: (description) => {
			const bannedLink = findBannedDescriptionLink(description ?? '')
			if (bannedLink) {
				return { valid: false, values: { fullUrl: bannedLink } }
			} else {
				return { valid: true }
			}
		},
		presentation: {
			message: messages.bannedLink,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'long-headers': {
		severity: 'error',
		evaluate: (description) => {
			const { longHeaders } = analyzeHeaderLength(description ?? '')
			if (longHeaders.length > 0) {
				return { valid: false, values: { count: longHeaders.length } }
			} else {
				return { valid: true }
			}
		},
		presentation: {
			message: messages.longHeaders,
			nag: { title: messages.shortenHeaders, ...commonNagPresentation },
		},
	},
	'missing-alt-text': {
		severity: 'warning',
		evaluate: (description) => ({
			valid: !analyzeImageContent(description ?? '').hasEmptyAltText,
		}),
		presentation: {
			message: messages.missingAltText,
			nag: { title: messages.addImageAltText, ...commonNagPresentation },
		},
	},
} satisfies ValidationRuleSet<DescriptionInput>

export function validateProjectDescription(
	description: DescriptionInput,
): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(description, projectDescriptionValidationRules))
}

export function getDescriptionNags(context: Pick<ProjectValidationContext, 'projectV3'>): Nag[] {
	const matches = evaluateRules(context.projectV3.description, projectDescriptionValidationRules)
	return toNags(
		projectRequiresEnglishText(context.projectV3)
			? matches
			: matches.filter(({ code }) => code !== 'project-description-non-english'),
	)
}
