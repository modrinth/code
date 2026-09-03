import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	title: { id: 'nags.review-permissions.title', defaultMessage: 'Check content permission' },
	description: {
		id: 'nags.review-permissions.description',
		defaultMessage:
			'Make sure you have provided proof of your permission to distribute any external content in your Modpack.',
	},
})

export const permissionNags = {
	'review-permissions': {
		title: messages.title,
		description: messages.description,
		destination: 'permissions',
	},
} satisfies NagDefinitions
