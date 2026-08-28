import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { findBlockedProjectContentLink } from '../../validators/links/detection.ts'
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
	shortenHeaders: {
		id: 'nags.long-headers.title',
		defaultMessage: 'Shorten headers',
	},
	ensureAccessibility: {
		id: 'nags.image-heavy-description.title',
		defaultMessage: 'Ensure accessibility',
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
			'Your description is {length, plural, one {# readable character} other {# readable characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create a clear and informative description.',
	},
	longHeaders: {
		id: 'nags.long-headers.description',
		defaultMessage:
			'{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
	},
	imageHeavy: {
		id: 'nags.image-heavy-description.description',
		defaultMessage:
			'Your Description should contain sufficient plain text or image alt-text, keeping it accessible to those using screen readers or with slow internet connections.',
	},
	missingAltText: {
		id: 'nags.missing-alt-text.description',
		defaultMessage:
			'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
	},
})

export const DESCRIPTION_MAX_PROFANITY_COUNT = 1
export const DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD = 0.05
export const MIN_DESCRIPTION_CHARS = 200
export const MAX_HEADER_LENGTH = 80
export const MIN_CHARS_PER_IMAGE = 60

export function analyzeHeaderLength(markdown: string): {
	hasLongHeaders: boolean
	longHeaders: string[]
} {
	if (!markdown) return { hasLongHeaders: false, longHeaders: [] }

	const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const headers = [...withoutCodeBlocks.matchAll(/^(#{1,3})\s+(.+)$/gm)]
	const longHeaders = headers
		.map((match) => match[2].trim())
		.filter((headerText) => {
			const sentences = headerText.split(/[.!?]+/g).filter((sentence) => sentence.trim().length > 0)
			return headerText.length > MAX_HEADER_LENGTH || sentences.length > 1
		})

	return { hasLongHeaders: longHeaders.length > 0, longHeaders }
}

export function countText(markdown: string): number {
	if (!markdown) return 0

	const withoutCode = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const withoutImagesAndLinks = withoutCode
		.replace(/!\[[^\]]*]\([^)]+\)/g, ' ')
		.replace(/\[[^\]]*]\([^)]+\)/g, ' ')
	const withoutHtml = withoutImagesAndLinks.replace(/<[^>]+>/g, ' ')
	const withoutMarkdownSyntax = withoutHtml
		.replace(/^(?:>[ \t]?)+/gm, '')
		.replace(/^#{1,6}\s+/gm, ' ')
		.replace(/[*_~`>-]/g, ' ')
		.replace(/\|/g, ' ')

	return withoutMarkdownSyntax.replace(/\s+/g, ' ').trim().length
}

export function analyzeImageContent(markdown: string): {
	imageHeavy: boolean
	hasEmptyAltText: boolean
} {
	if (!markdown) return { imageHeavy: false, hasEmptyAltText: false }

	const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const images = [...withoutCodeBlocks.matchAll(/!\[([^\]]*)\]\([^)]+\)/g)]
	const htmlImages = [...withoutCodeBlocks.matchAll(/<img[^>]*>/gi)]
	const totalImages = images.length + htmlImages.length
	if (totalImages === 0) return { imageHeavy: false, hasEmptyAltText: false }

	const textLength = countText(withoutCodeBlocks)
	const recommendedTextLength = MIN_CHARS_PER_IMAGE * totalImages
	const imageHeavy =
		recommendedTextLength > MIN_DESCRIPTION_CHARS && textLength < recommendedTextLength
	const hasEmptyAltText =
		images.some((match) => !match[1]?.trim()) ||
		htmlImages.some((match) => {
			const altMatch = match[0].match(/alt\s*=\s*["']([^"']*)["']/i)
			return !altMatch || !altMatch[1]?.trim()
		})

	return { imageHeavy, hasEmptyAltText }
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
	'project-description-banned-link': {
		severity: 'error',
		evaluate: (description) => {
			const blockedLink = findBlockedProjectContentLink(description ?? '')
			return blockedLink ? { valid: false, values: { fullUrl: blockedLink.url } } : { valid: true }
		},
		presentation: {
			message: messages.bannedLink,
			nag: { title: messages.fixDescription, ...commonNagPresentation },
		},
	},
	'description-too-short': {
		severity: 'warning',
		evaluate: (description) => {
			const normalized = normalizeProjectFieldText(description ?? '')
			if (!normalized) return { valid: true }
			const length = countText(normalized)
			return length < MIN_DESCRIPTION_CHARS
				? { valid: false, values: { length, minChars: MIN_DESCRIPTION_CHARS } }
				: { valid: true }
		},
		presentation: {
			message: messages.tooShort,
			nag: { title: messages.expandDescription, ...commonNagPresentation },
		},
	},
	'long-headers': {
		severity: 'warning',
		evaluate: (description) => {
			const { longHeaders } = analyzeHeaderLength(description ?? '')
			return longHeaders.length > 0
				? { valid: false, values: { count: longHeaders.length } }
				: { valid: true }
		},
		presentation: {
			message: messages.longHeaders,
			nag: { title: messages.shortenHeaders, ...commonNagPresentation },
		},
	},
	'image-heavy-description': {
		severity: 'warning',
		evaluate: (description) => ({
			valid: !analyzeImageContent(description ?? '').imageHeavy,
		}),
		presentation: {
			message: messages.imageHeavy,
			nag: { title: messages.ensureAccessibility, ...commonNagPresentation },
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
	return toNags(evaluateRules(context.projectV3.description, projectDescriptionValidationRules))
}
