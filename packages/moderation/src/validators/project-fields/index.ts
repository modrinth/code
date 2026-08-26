import LinkifyIt from 'linkify-it'

import { getNonStandardTextRatio, validateNonStandardText } from '../non-standard-text/index.ts'
import { validateProfanity } from '../profanity/index.ts'

export interface ProjectFieldMessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

function defineMessages<T extends Record<string, ProjectFieldMessageDescriptor>>(
	descriptors: T,
): T {
	return descriptors
}

export interface ProjectTextValidationResult {
	code: ProjectTextValidationCode
	severity: 'warn' | 'error'
	message: ProjectFieldMessageDescriptor
	values?: Record<string, unknown>
}

export type ProjectTextValidationCode =
	| 'text-slur'
	| 'text-profanity'
	| 'text-non-standard'
	| 'title-game-version'
	| 'title-loader'
	| 'title-minecraft-branding'
	| 'summary-link'
	| 'summary-matches-title'
	| 'summary-too-short'
	| 'summary-special-formatting'
	| 'description-required'
	| 'description-too-short'
	| 'description-long-headers'
	| 'description-image-heavy'
	| 'description-missing-alt-text'

export interface ProjectTextValidationOptions {
	maxProfanityCount?: number
	nonStandardTextFailureThreshold?: number
}

export const DESCRIPTION_MAX_PROFANITY_COUNT = 1
export const DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD = 0.05
export const MIN_DESCRIPTION_CHARS = 200
export const MAX_HEADER_LENGTH = 80
export const MIN_CHARS_PER_IMAGE = 60
export const MIN_SUMMARY_CHARS = 30

const messages = defineMessages({
	slur: {
		id: 'project.text-validation.slur',
		defaultMessage: 'The detected slur “{value}” is not allowed.',
	},
	profanity: {
		id: 'project.text-validation.profanity',
		defaultMessage: 'The detected profanity “{value}” is not allowed.',
	},
	descriptionProfanity: {
		id: 'project.text-validation.description-profanity',
		defaultMessage: 'Excessive profanity is not allowed. Detected: {values}',
	},
	nonStandardText: {
		id: 'project.text-validation.non-standard-text',
		defaultMessage: 'Non-standard text characters are not allowed.',
	},
	titleGameVersion: {
		id: 'project.text-validation.title-game-version',
		defaultMessage: 'Project titles should not include the Minecraft version “{value}”.',
	},
	titleLoader: {
		id: 'project.text-validation.title-loader',
		defaultMessage: 'Project titles should not include the loader “{value}”.',
	},
	titleMinecraftBranding: {
		id: 'nags.minecraft-title-clause.description',
		defaultMessage:
			'Projects must not use Minecraft\'s branding or include "Minecraft" as a significant part of the name.',
	},
	summaryLink: {
		id: 'project.text-validation.summary-link',
		defaultMessage: 'Links should not be included in project summaries.',
	},
	summaryMatchesTitle: {
		id: 'project.text-validation.summary-matches-title',
		defaultMessage: 'A project summary should not be the same as its title.',
	},
	summaryTooShort: {
		id: 'project.text-validation.summary-too-short',
		defaultMessage:
			'Your summary is {length, plural, one {# character} other {# characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create an informative and enticing summary.',
	},
	summarySpecialFormatting: {
		id: 'nags.summary-special-formatting.description',
		defaultMessage:
			'Your summary should not contain formatting, line breaks, or special characters, since the summary will only display plain text.',
	},
	descriptionRequired: {
		id: 'nags.add-description.description',
		defaultMessage:
			"A description that clearly describes the project's purpose and function is required.",
	},
	descriptionTooShort: {
		id: 'nags.description-too-short.description',
		defaultMessage:
			'Your description is {length, plural, one {# readable character} other {# readable characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create a clear and informative description.',
	},
	descriptionLongHeaders: {
		id: 'nags.long-headers.description',
		defaultMessage:
			'{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
	},
	descriptionImageHeavy: {
		id: 'nags.image-heavy-description.description',
		defaultMessage:
			'Your Description should contain sufficient plain text or image alt-text, keeping it accessible to those using screen readers or with slow internet connections.',
	},
	descriptionMissingAltText: {
		id: 'nags.missing-alt-text.description',
		defaultMessage:
			'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
	},
})

const titleMetadataMessages = {
	'game-version': messages.titleGameVersion,
	loader: messages.titleLoader,
}

export type ProjectTitleMetadataKind = 'game-version' | 'loader'

export interface ProjectTitleMetadata {
	gameVersions: readonly string[]
	loaders: readonly string[]
}

export interface ProjectTitleMetadataMatch {
	kind: ProjectTitleMetadataKind
	value: string
}

const linkify = new LinkifyIt({
	fuzzyEmail: false,
	fuzzyIP: true,
	fuzzyLink: true,
})

function normalizeForSearch(value: string) {
	return value.normalize('NFC').toLowerCase()
}

export function findProjectTitleMetadata(
	title: string,
	metadata: ProjectTitleMetadata,
): ProjectTitleMetadataMatch | null {
	const normalizedTitle = normalizeForSearch(title)
	const groups: ReadonlyArray<readonly [ProjectTitleMetadataKind, readonly string[]]> = [
		['game-version', metadata.gameVersions],
		['loader', metadata.loaders],
	]

	for (const [kind, values] of groups) {
		for (const value of values) {
			const normalizedValue = normalizeForSearch(value.trim())
			if (normalizedValue && normalizedTitle.includes(normalizedValue)) {
				return { kind, value }
			}
		}
	}

	return null
}

export function normalizeProjectFieldText(value: string) {
	return value.trim().normalize('NFC')
}

export function projectSummaryMatchesTitle(summary: string, title: string) {
	const normalizedSummary = normalizeProjectFieldText(summary)
	const normalizedTitle = normalizeProjectFieldText(title)

	return normalizedSummary.length > 0 && normalizedSummary === normalizedTitle
}

export function extractProjectLinks(text: string) {
	const matches = linkify.match(text) ?? []
	return [...new Set(matches.map((match) => match.url))]
}

export function containsProjectLinkOrIp(text: string) {
	return linkify.test(text)
}

export function hasProjectSummaryFormatting(summary: string) {
	return Boolean(
		summary.match(/# .*/g) ||
		summary.match(/---/g) ||
		summary.match(/\n/g) ||
		summary.match(/`.*`/g) ||
		summary.match(/\*.*\*/g) ||
		summary.match(/_.*_/g) ||
		summary.match(/~~.*~~/g) ||
		summary.match(/```/g) ||
		summary.match(/> /g),
	)
}

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

	return {
		hasLongHeaders: longHeaders.length > 0,
		longHeaders,
	}
}

export function countText(markdown: string): number {
	if (!markdown) return 0

	const withoutCode = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
	const withoutImagesAndLinks = withoutCode
		.replace(/!\[[^\]]*]\([^)]+\)/g, ' ')
		.replace(/\[[^\]]*]\([^)]+\)/g, ' ')
	const withoutHtml = withoutImagesAndLinks.replace(/<[^>]+>/g, ' ')
	const withoutMarkdownSyntax = withoutHtml
		.replace(/^>{1}\s?.*$/gm, ' ')
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

export function validateProjectText(
	text: string | null | undefined,
	options: ProjectTextValidationOptions = {},
): ProjectTextValidationResult[] {
	if (!text) return []

	const profanity = validateProfanity(text)
	const slurMatch = profanity.matches.find((match) => match.kind === 'slur')
	if (slurMatch) {
		return [
			{
				code: 'text-slur',
				severity: 'error',
				message: messages.slur,
				values: { value: slurMatch.rawText },
			},
		]
	}

	const maxProfanityCount = options.maxProfanityCount ?? 0
	if (!Number.isInteger(maxProfanityCount) || maxProfanityCount < 0) {
		throw new Error('Maximum profanity count must be a non-negative integer')
	}
	const profanityMatch = profanity.matches.filter((match) => match.kind === 'profanity')[
		maxProfanityCount
	]
	if (profanityMatch) {
		return [
			{
				code: 'text-profanity',
				severity: 'error',
				message: messages.profanity,
				values: { value: profanityMatch.rawText },
			},
		]
	}

	const nonStandardText = validateNonStandardText(text)
	const nonStandardTextFailureThreshold = options.nonStandardTextFailureThreshold ?? 0
	if (
		!nonStandardText.valid &&
		getNonStandardTextRatio(text, nonStandardText) >= nonStandardTextFailureThreshold
	) {
		return [{ code: 'text-non-standard', severity: 'error', message: messages.nonStandardText }]
	}

	return []
}

export function validateProjectTitle(
	text: string | null | undefined,
	metadata: ProjectTitleMetadata,
): ProjectTextValidationResult[] {
	const results = validateProjectText(text)
	if (results.length > 0 || !text) return results

	const match = findProjectTitleMetadata(text, metadata)
	if (match) {
		results.push({
			code: match.kind === 'game-version' ? 'title-game-version' : 'title-loader',
			severity: 'warn',
			message: titleMetadataMessages[match.kind],
			values: { value: match.value },
		})
	}

	const normalizedTitle = normalizeProjectFieldText(text).toLowerCase()
	const wordsInTitle = normalizedTitle.split(/\s+/).filter(Boolean)
	if (normalizedTitle.includes('minecraft') && wordsInTitle.length <= 3) {
		results.push({
			code: 'title-minecraft-branding',
			severity: 'warn',
			message: messages.titleMinecraftBranding,
		})
	}

	return results
}

export function validateProjectSummary(
	summary: string | null | undefined,
	title: string | null | undefined,
): ProjectTextValidationResult[] {
	const results = validateProjectText(summary)
	if (results.length > 0 || !summary) return results

	if (containsProjectLinkOrIp(summary)) {
		return [{ code: 'summary-link', severity: 'warn', message: messages.summaryLink }]
	}

	if (title && projectSummaryMatchesTitle(summary, title)) {
		return [
			{
				code: 'summary-matches-title',
				severity: 'warn',
				message: messages.summaryMatchesTitle,
			},
		]
	}

	const length = normalizeProjectFieldText(summary).length
	if (length < MIN_SUMMARY_CHARS) {
		results.push({
			code: 'summary-too-short',
			severity: 'warn',
			message: messages.summaryTooShort,
			values: { length, minChars: MIN_SUMMARY_CHARS },
		})
	}

	if (hasProjectSummaryFormatting(summary)) {
		results.push({
			code: 'summary-special-formatting',
			severity: 'warn',
			message: messages.summarySpecialFormatting,
		})
	}

	return results
}

export function validateProjectDescription(
	description: string | null | undefined,
): ProjectTextValidationResult[] {
	const results = validateProjectText(description, {
		maxProfanityCount: DESCRIPTION_MAX_PROFANITY_COUNT,
		nonStandardTextFailureThreshold: DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD,
	})
	if (results[0]?.code === 'text-profanity') {
		const detectedValues = validateProfanity(description ?? '')
			.matches.filter((match) => match.kind === 'profanity')
			.map((match) => `"${match.rawText}"`)
			.join(', ')

		return [
			{
				...results[0],
				message: messages.descriptionProfanity,
				values: { values: detectedValues },
			},
		]
	}
	if (results.length > 0) return results

	const normalizedDescription = normalizeProjectFieldText(description ?? '')
	if (!normalizedDescription) {
		return [
			{
				code: 'description-required',
				severity: 'error',
				message: messages.descriptionRequired,
			},
		]
	}

	const readableLength = countText(normalizedDescription)
	if (readableLength < MIN_DESCRIPTION_CHARS) {
		results.push({
			code: 'description-too-short',
			severity: 'warn',
			message: messages.descriptionTooShort,
			values: { length: readableLength, minChars: MIN_DESCRIPTION_CHARS },
		})
	}

	const { hasLongHeaders, longHeaders } = analyzeHeaderLength(normalizedDescription)
	if (hasLongHeaders) {
		results.push({
			code: 'description-long-headers',
			severity: 'warn',
			message: messages.descriptionLongHeaders,
			values: { count: longHeaders.length },
		})
	}

	const { imageHeavy, hasEmptyAltText } = analyzeImageContent(normalizedDescription)
	if (imageHeavy) {
		results.push({
			code: 'description-image-heavy',
			severity: 'warn',
			message: messages.descriptionImageHeavy,
		})
	}
	if (hasEmptyAltText) {
		results.push({
			code: 'description-missing-alt-text',
			severity: 'warn',
			message: messages.descriptionMissingAltText,
		})
	}

	return results
}
