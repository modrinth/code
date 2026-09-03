import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	title: { id: 'nags.add-icon.title', defaultMessage: 'Add an icon' },
	description: {
		id: 'nags.add-icon.description',
		defaultMessage:
			'Adding a unique, relevant, and engaging icon makes your project identifiable and helps it stand out.',
	},
})

export const iconNags = {
	'add-icon': { title: messages.title, description: messages.description, destination: 'general' },
} satisfies NagDefinitions
