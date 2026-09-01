import { md } from '@modrinth/utils/parse.ts'

import type { ProjectValidationContext } from '../types/nags.ts'
import {
	validateEnglishSummaryText,
	validateEnglishText,
	validateEnglishTextBlocks,
} from '../validators/language/index.ts'
import {
	getNonStandardTextRatio,
	validateNonStandardText,
} from '../validators/non-standard-text/index.ts'
import { validateProfanity } from '../validators/profanity/index.ts'
import type { ValidationRuleEvaluation } from './types.ts'

const projectPlainTextMarkdown = md({ linkify: false })
const allowedPlainTextBlockTokenTypes = new Set(['paragraph_open', 'inline', 'paragraph_close'])
const allowedPlainTextInlineTokenTypes = new Set(['text', 'softbreak', 'hardbreak'])

export function hasProjectTextFormatting(text: string) {
	return projectPlainTextMarkdown.parse(text, {}).some((token) => {
		if (!allowedPlainTextBlockTokenTypes.has(token.type)) return true

		return (
			token.children?.some((child) => !allowedPlainTextInlineTokenTypes.has(child.type)) ?? false
		)
	})
}

const pairedHtmlTagPattern = /<([a-z][\w:-]*)\b[^>]*>[\s\S]*?<\/\1\s*>/i

function hasExplicitlyClosedHtmlElement(html: string) {
	return pairedHtmlTagPattern.test(html)
}

export function hasProjectTextHtmlFormatting(text: string) {
	const tokens = projectPlainTextMarkdown.parse(text, {})

	for (const token of tokens) {
		if (token.type === 'html_block' && hasExplicitlyClosedHtmlElement(token.content)) return true
		if (!token.children?.some((child) => child.type === 'html_inline')) continue

		const inlineHtml = token.children
			.filter((child) => child.type !== 'code_inline')
			.map((child) => child.content)
			.join('')

		if (hasExplicitlyClosedHtmlElement(inlineHtml)) return true
	}

	return false
}

export function normalizeProjectFieldText(value: string) {
	return value.trim().normalize('NFC')
}

export function projectRequiresEnglishText(project: ProjectValidationContext['projectV3']) {
	return (
		(!project.minecraft_java_server &&
			!project.categories?.includes('locale') &&
			!project.additional_categories?.includes('locale')) ||
		project.minecraft_server?.languages?.includes('en') === true
	)
}

export function evaluateSlur(text: string): ValidationRuleEvaluation {
	const match = validateProfanity(text).matches.find((match) => match.kind === 'slur')
	return match ? { valid: false, values: { value: match.rawText } } : { valid: true }
}

export function evaluateProfanity(text: string, maxProfanityCount = 0): ValidationRuleEvaluation {
	if (!Number.isInteger(maxProfanityCount) || maxProfanityCount < 0) {
		throw new Error('Maximum profanity count must be a non-negative integer')
	}

	const match = validateProfanity(text).matches.filter((match) => match.kind === 'profanity')[
		maxProfanityCount
	]
	return match ? { valid: false, values: { value: match.rawText } } : { valid: true }
}

export function evaluateNonStandardText(
	text: string,
	failureThreshold = 0,
): ValidationRuleEvaluation {
	const result = validateNonStandardText(text)
	return {
		valid: result.valid || getNonStandardTextRatio(text, result) < failureThreshold,
	}
}

export function evaluateEnglishText(text: string): ValidationRuleEvaluation {
	const result = validateEnglishText(text)
	return { valid: result.valid }
}

export function evaluateEnglishTextBlocks(blocks: string[]): ValidationRuleEvaluation {
	const result = validateEnglishTextBlocks(blocks)
	return { valid: result.valid }
}

export function evaluateEnglishSummaryText(text: string): ValidationRuleEvaluation {
	const result = validateEnglishSummaryText(text)
	return { valid: result.valid }
}
