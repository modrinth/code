import { validateNonStandardText, validateProfanity } from '@modrinth/moderation'
import { defineMessages, type MessageDescriptor } from '@modrinth/ui'

export interface ProjectTextValidationResult {
	severity: 'error'
	message: MessageDescriptor
}

const messages = defineMessages({
	slur: {
		id: 'project.text-validation.slur',
		defaultMessage: 'Slurs are not allowed.',
	},
	profanity: {
		id: 'project.text-validation.profanity',
		defaultMessage: 'Profanity is not allowed.',
	},
	nonStandardText: {
		id: 'project.text-validation.non-standard-text',
		defaultMessage: 'Non-standard text characters are not allowed.',
	},
})

export function validateProjectText(
	text: string | null | undefined,
): ProjectTextValidationResult | null {
	if (!text) return null

	const profanity = validateProfanity(text)
	if (profanity.slurCount > 0) {
		return { severity: 'error', message: messages.slur }
	}
	if (profanity.profanityCount > 0) {
		return { severity: 'error', message: messages.profanity }
	}

	if (!validateNonStandardText(text).valid) {
		return { severity: 'error', message: messages.nonStandardText }
	}

	return null
}
