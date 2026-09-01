import type { Labrinth } from '@modrinth/api-client'
import { defineMessages } from '@modrinth/ui/i18n'
import { formatProjectTypeSentence } from '@modrinth/ui/src/utils/common-messages.ts'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { hasProjectTextHtmlFormatting } from '../text.ts'
import { toFieldMessages } from '../to-field-messages.ts'
import { toNags } from '../to-nags.ts'
import type { FieldValidationMessage, ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	title: {
		id: 'nags.check-disclosures.title',
		defaultMessage: 'Check content disclosures',
	},
	description: {
		id: 'nags.check-disclosures.description',
		defaultMessage:
			'Make sure users are aware of any important details by filling in content disclosures that apply to your {type}.',
	},
	removeFormatting: {
		id: 'nags.disclosures-special-formatting.title',
		defaultMessage: 'Remove HTML from content disclosures',
	},
	specialFormatting: {
		id: 'nags.disclosures-special-formatting.description',
		defaultMessage: 'Content disclosures cannot contain HTML, as they display in plain text.',
	},
})

function getDisclosureText(disclosure: Labrinth.Projects.v3.ProjectDisclosure): string[] {
	switch (disclosure.type) {
		case 'ai_content':
		case 'advertisements':
		case 'epilepsy_triggers':
		case 'archived':
			return disclosure.note ? [disclosure.note] : []
		case 'system_interactions':
			return disclosure.note ? [disclosure.note] : []
		case 'telemetry':
			return disclosure.data_collected
		case 'derivative_work':
			return disclosure.sources.flatMap((source) =>
				source.note ? [source.label, source.note] : [source.label],
			)
		case 'paid_features':
			return disclosure.features
	}
}

export const projectDisclosureTextValidationRules = {
	'disclosures-special-formatting': {
		severity: 'error',
		evaluate: (disclosures) => ({
			valid: !disclosures.some((disclosure) =>
				getDisclosureText(disclosure).some(hasProjectTextHtmlFormatting),
			),
		}),
		presentation: {
			message: messages.specialFormatting,
			nag: { title: messages.removeFormatting, destination: 'disclosures' },
		},
	},
} satisfies ValidationRuleSet<readonly Labrinth.Projects.v3.ProjectDisclosure[]>

export function validateProjectDisclosures(
	disclosures: readonly Labrinth.Projects.v3.ProjectDisclosure[],
): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(disclosures, projectDisclosureTextValidationRules))
}

export const projectDisclosureValidationRules = {
	'check-disclosures': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid: false,
			values: { projectType: context.project.project_type },
		}),
		presentation: {
			message: messages.description,
			nag: {
				title: messages.title,
				destination: 'disclosures',
				formatValues: (values, formatMessage) => ({
					type: formatProjectTypeSentence(formatMessage, String(values.projectType)),
				}),
			},
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getDisclosureNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectDisclosureValidationRules))
}
