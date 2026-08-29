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

export function normalizeProjectFieldText(value: string) {
	return value.trim().normalize('NFC')
}

export function projectRequiresEnglishText(
	project: Pick<
		ProjectValidationContext['projectV3'],
		'minecraft_java_server' | 'minecraft_server'
	>,
) {
	return (
		!project.minecraft_java_server || project.minecraft_server?.languages?.includes('en') === true
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
