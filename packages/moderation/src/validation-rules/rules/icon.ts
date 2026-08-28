import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	title: {
		id: 'nags.add-icon.title',
		defaultMessage: 'Add an icon',
	},
	description: {
		id: 'nags.add-icon.description',
		defaultMessage:
			'Adding a unique, relevant, and engaging icon makes your project identifiable and helps it stand out.',
	},
})

export const projectIconValidationRules = {
	'add-icon': {
		severity: 'suggestion',
		evaluate: (context) => ({ valid: Boolean(context.project.icon_url) }),
		presentation: {
			message: messages.description,
			nag: { title: messages.title, destination: 'general' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getIconNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectIconValidationRules))
}
