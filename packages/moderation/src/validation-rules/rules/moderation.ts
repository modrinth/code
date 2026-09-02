import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	title: {
		id: 'nags.moderator-feedback.title',
		defaultMessage: `Review feedback`,
	},
	description: {
		id: 'nags.moderator-feedback.description',
		defaultMessage: `Review and address all concerns from the moderation team before resubmitting.`,
	},
})

export const projectModerationValidationRules = {
	'moderator-feedback': {
		severity: 'warning',
		evaluate: (context) => ({
			valid: !context.tags.rejectedStatuses.includes(context.project.status),
		}),
		presentation: {
			message: messages.description,
			nag: { title: messages.title, destination: 'moderation' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getModerationNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectModerationValidationRules))
}
