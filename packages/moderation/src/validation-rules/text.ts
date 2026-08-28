import {
	getNonStandardTextRatio,
	validateNonStandardText,
} from '../validators/non-standard-text/index.ts'
import { validateProfanity } from '../validators/profanity/index.ts'
import type { ValidationRuleEvaluation } from './types.ts'

export function normalizeProjectFieldText(value: string) {
	return value.trim().normalize('NFC')
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
