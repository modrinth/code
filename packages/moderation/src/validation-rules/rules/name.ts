import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, NagContext } from '../../types/nags.ts'
import { validateNonStandardText } from '../../validators/non-standard-text/index.ts'
import { validateProfanity } from '../../validators/profanity/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toFieldMessages } from '../to-field-messages.ts'
import { toNags } from '../to-nags.ts'
import type { FieldValidationMessage, ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	fixName: {
		id: 'nags.invalid-project-name.title',
		defaultMessage: 'Fix the project name',
	},
	fixVersion: {
		id: 'nags.project-name-version.title',
		defaultMessage: 'Fix project name',
	},
	avoidBrandInfringement: {
		id: 'nags.minecraft-title-clause.title',
		defaultMessage: 'Avoid brand infringement',
	},
	editName: {
		id: 'nags.edit-title.title',
		defaultMessage: 'Edit title',
	},
	slur: {
		id: 'nags.project-name-slur.description',
		defaultMessage: 'Your project name cannot contain any slurs. Detected: “{value}”.',
	},
	profanity: {
		id: 'nags.project-name-profanity.description',
		defaultMessage: 'Your project name cannot contain profanity. Detected: “{value}”.',
	},
	nonStandardText: {
		id: 'nags.project-name-non-standard-text.description',
		defaultMessage: 'Non-standard text characters, such as “₮ɆӾ₮”, are not allowed.',
	},
	versionNumber: {
		id: 'project.text-validation.title-version-number',
		defaultMessage: 'Names are not allowed to include version numbers.',
	},
	minecraftBranding: {
		id: 'nags.minecraft-title-clause.description',
		defaultMessage:
			'Projects must not use Minecraft\'s branding or include "Minecraft" as a significant part of the name.',
	},
})

export const projectNameValidationRules = {
	'project-name-slur': {
		severity: 'error',
		evaluate: (projectName) => {
			const match = validateProfanity(projectName).matches.find((match) => match.kind === 'slur')
			if (match) {
				return { valid: false, values: { value: match.rawText } }
			} else {
				return { valid: true }
			}
		},
		presentation: {
			message: messages.slur,
			nag: {
				title: messages.fixName,
				destination: 'general',
				linkTitle: messages.editName,
			},
		},
	},
	'project-name-profanity': {
		severity: 'error',
		evaluate: (projectName) => {
			const match = validateProfanity(projectName).matches.find(
				(match) => match.kind === 'profanity',
			)
			if (match) {
				return { valid: false, values: { value: match.rawText } }
			} else {
				return { valid: true }
			}
		},
		presentation: {
			message: messages.profanity,
			nag: {
				title: messages.fixName,
				destination: 'general',
				linkTitle: messages.editName,
			},
		},
	},
	'project-name-non-standard-text': {
		severity: 'error',
		evaluate: (projectName) => ({ valid: validateNonStandardText(projectName).valid }),
		presentation: {
			message: messages.nonStandardText,
			nag: {
				title: messages.fixName,
				destination: 'general',
				linkTitle: messages.editName,
			},
		},
	},
	'project-name-version': {
		severity: 'error',
		evaluate: (projectName) => {
			const normalizedName = projectName.normalize('NFC').toLowerCase()
			const includesVersionNumber = [...normalizedName.matchAll(/\d+(?:\.\d+)+/g)].some((match) => {
				const textAfterVersion = normalizedName.slice((match.index ?? 0) + match[0].length)
				return !/\b(?:port|fork)\b/.test(textAfterVersion)
			})
			return { valid: !includesVersionNumber }
		},
		presentation: {
			message: messages.versionNumber,
			nag: {
				title: messages.fixVersion,
				destination: 'general',
				linkTitle: messages.editName,
			},
		},
	},
	'minecraft-title-clause': {
		severity: 'warning',
		evaluate: (projectName) => {
			const normalizedName = projectName.normalize('NFC').toLowerCase()
			const words = normalizedName.split(/\s+/).filter(Boolean)
			return {
				valid: !(normalizedName.includes('minecraft') && words.length <= 3),
			}
		},
		presentation: {
			message: messages.minecraftBranding,
			nag: {
				title: messages.avoidBrandInfringement,
				destination: 'general',
				linkTitle: messages.editName,
			},
		},
	},
} satisfies ValidationRuleSet<string>

export function validateProjectNameField(name: string): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(name, projectNameValidationRules))
}

export function getNameNags(context: Pick<NagContext, 'projectV3'>): Nag[] {
	return toNags(evaluateRules(context.projectV3.name, projectNameValidationRules))
}
