import type { FieldValidationMessage, ValidationRuleMatch } from './types.ts'

export function toFieldMessages(matches: ValidationRuleMatch[]): FieldValidationMessage[] {
	return matches.map((match) => ({
		code: match.code,
		severity: match.rule.severity,
		message: match.message,
		values: Object.keys(match.values).length > 0 ? match.values : undefined,
	}))
}
