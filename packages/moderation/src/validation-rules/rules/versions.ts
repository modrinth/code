import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	title: {
		id: 'nags.upload-version.title',
		defaultMessage: 'Upload a version',
	},
	description: {
		id: 'nags.upload-version.description',
		defaultMessage: 'At least one version is required for a project to be submitted for review.',
	},
})

export const projectVersionValidationRules = {
	'upload-version': {
		severity: 'error',
		evaluate: (context) => ({
			valid: context.projectV3.versions.length > 0 || Boolean(context.projectV3.minecraft_server),
		}),
		presentation: {
			message: messages.description,
			nag: { title: messages.title, destination: 'versions' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getVersionNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectVersionValidationRules))
}
