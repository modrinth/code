import { formatProjectTypeSentence } from '@modrinth/ui'
import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

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
})

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
