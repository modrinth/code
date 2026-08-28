import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	title: {
		id: 'nags.review-permissions.title',
		defaultMessage: 'Review external permissions',
	},
	description: {
		id: 'nags.review-permissions.description',
		defaultMessage:
			'Make sure you have provided proof of your permission to distribute any external content in your Modpack.',
	},
})

export const projectPermissionsValidationRules = {
	'review-permissions': {
		severity: 'error',
		evaluate: (context) => ({
			valid: !context.versions.some(
				(version) => (version.files_missing_attribution?.length ?? 0) >= 1,
			),
		}),
		presentation: {
			message: messages.description,
			nag: { title: messages.title, destination: 'permissions' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getPermissionsNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectPermissionsValidationRules))
}
