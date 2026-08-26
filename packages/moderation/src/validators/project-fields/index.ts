import LinkifyIt from 'linkify-it'

import { getNonStandardTextRatio, validateNonStandardText } from '../non-standard-text/index.ts'
import { validateProfanity } from '../profanity/index.ts'

export interface ProjectFieldMessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

function defineMessages<T extends Record<string, ProjectFieldMessageDescriptor>>(descriptors: T): T {
	return descriptors
}

export interface ProjectTextValidationResult {
	severity: 'error'
	message: ProjectFieldMessageDescriptor
	values?: Record<string, unknown>
}

export interface ProjectTextValidationOptions {
	nonStandardTextFailureThreshold?: number
}

export const DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD = 0.05

const messages = defineMessages({
	slur: {
		id: 'project.text-validation.slur',
		defaultMessage: 'Slurs are not allowed.',
	},
	profanity: {
		id: 'project.text-validation.profanity',
		defaultMessage: 'Profanity is not allowed.',
	},
	nonStandardText: {
		id: 'project.text-validation.non-standard-text',
		defaultMessage: 'Non-standard text characters are not allowed.',
	},
	titleGameVersion: {
		id: 'project.text-validation.title-game-version',
		defaultMessage: 'Project titles cannot include the Minecraft version “{value}”.',
	},
	titleLoader: {
		id: 'project.text-validation.title-loader',
		defaultMessage: 'Project titles cannot include the loader “{value}”.',
	},
	summaryLink: {
		id: 'project.text-validation.summary-link',
		defaultMessage: 'Links are not allowed in project summaries.',
	},
	summaryMatchesTitle: {
		id: 'project.text-validation.summary-matches-title',
		defaultMessage: 'A project summary cannot be the same as its title.',
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

export function validateProjectText(
	text: string | null | undefined,
	options: ProjectTextValidationOptions = {},
): ProjectTextValidationResult | null {
	if (!text) return null

	const profanity = validateProfanity(text)
	if (profanity.slurCount > 0) {
		return { severity: 'error', message: messages.slur }
	}
	if (profanity.profanityCount > 0) {
		return { severity: 'error', message: messages.profanity }
	}

	const nonStandardText = validateNonStandardText(text)
	const nonStandardTextFailureThreshold = options.nonStandardTextFailureThreshold ?? 0
	if (
		!nonStandardText.valid &&
		getNonStandardTextRatio(text, nonStandardText) >= nonStandardTextFailureThreshold
	) {
		return { severity: 'error', message: messages.nonStandardText }
	}

	return null
}

export function validateProjectTitle(
	text: string | null | undefined,
	metadata: ProjectTitleMetadata,
): ProjectTextValidationResult | null {
	const textValidation = validateProjectText(text)
	if (textValidation || !text) return textValidation

	const match = findProjectTitleMetadata(text, metadata)
	if (!match) return null

	return {
		severity: 'error',
		message: titleMetadataMessages[match.kind],
		values: { value: match.value },
	}
}

export function validateProjectSummary(
	summary: string | null | undefined,
	title: string | null | undefined,
): ProjectTextValidationResult | null {
	const textValidation = validateProjectText(summary)
	if (textValidation || !summary) return textValidation

	if (containsProjectLinkOrIp(summary)) {
		return { severity: 'error', message: messages.summaryLink }
	}

	if (title && projectSummaryMatchesTitle(summary, title)) {
		return { severity: 'error', message: messages.summaryMatchesTitle }
	}

	return null
}

export function validateProjectDescription(
	description: string | null | undefined,
): ProjectTextValidationResult | null {
	return validateProjectText(description, {
		nonStandardTextFailureThreshold: DESCRIPTION_NON_STANDARD_TEXT_FAILURE_THRESHOLD,
	})
}
