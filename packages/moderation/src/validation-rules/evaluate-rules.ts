import type { ValidationRuleMatch, ValidationRuleSet } from './types.ts'

// evaluate a set of rules and return the failed rules
export function evaluateRules<Input, Rules extends ValidationRuleSet<Input>>(
	input: Input,
	rules: Rules,
): ValidationRuleMatch<Extract<keyof Rules, string>>[] {
	type RuleCode = Extract<keyof Rules, string>

	const codes = Object.keys(rules) as RuleCode[]
	return codes.flatMap<ValidationRuleMatch<RuleCode>>((code) => {
		const rule = rules[code]
		const result = rule.evaluate(input)
		if (result.valid) return []

		return [
			{
				code,
				message: result.message ?? rule.presentation.message,
				rule,
				values: result.values ?? {},
			},
		]
	})
}
