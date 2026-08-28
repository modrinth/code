import type { MessageDescriptor, VIntlFormatters } from '@modrinth/ui'

import type { NagDestinationId } from '../types/nags.ts'

export type ValidationRuleSeverity = 'error' | 'warning' | 'suggestion'

export type ValidationRuleValues = Record<string, string | number | boolean>

export type ValidationRuleValueFormatter = (
	values: ValidationRuleValues,
	formatMessage: VIntlFormatters['formatMessage'],
) => ValidationRuleValues

export interface ValidationRulePresentation {
	message: MessageDescriptor
	nag: {
		title: MessageDescriptor
		destination?: NagDestinationId
		linkTitle?: MessageDescriptor
		formatValues?: ValidationRuleValueFormatter
	}
}

export interface ValidationRuleDefinition {
	severity: ValidationRuleSeverity
	presentation: ValidationRulePresentation
}

export type ValidationRuleEvaluation =
	| { valid: true }
	| { valid: false; message?: MessageDescriptor; values?: ValidationRuleValues }

export interface ValidationRule<Input> extends ValidationRuleDefinition {
	evaluate: (input: Input) => ValidationRuleEvaluation
}

export type ValidationRuleSet<Input> = Readonly<Record<string, ValidationRule<Input>>>

export interface ValidationRuleMatch<Code extends string = string> {
	code: Code
	message: MessageDescriptor
	rule: ValidationRuleDefinition
	values: ValidationRuleValues
}

export interface FieldValidationMessage {
	code: string
	severity: ValidationRuleSeverity
	message: MessageDescriptor
	values?: ValidationRuleValues
}
