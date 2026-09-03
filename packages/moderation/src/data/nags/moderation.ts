import { defineMessages } from '@modrinth/ui/i18n'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	title: { id: 'nags.moderator-feedback.title', defaultMessage: 'Review feedback' },
	description: {
		id: 'nags.moderator-feedback.description',
		defaultMessage: 'Review and address all concerns from the moderation team before resubmitting.',
	},
})

export const moderationNags = {
	'moderator-feedback': {
		title: messages.title,
		description: messages.description,
		destination: 'moderation',
	},
} satisfies NagDefinitions
