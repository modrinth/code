import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	environmentTitle: {
		id: 'nags.select-environment.title',
		defaultMessage: 'Select an environment',
	},
	environment: {
		id: 'nags.select-environment.description',
		defaultMessage: 'Specify the environment where your project can run.',
	},
	uploadTitle: { id: 'nags.upload-version.title', defaultMessage: 'Upload a version' },
	upload: {
		id: 'nags.upload-version.description',
		defaultMessage: 'At least one version is required for a project to be submitted for review.',
	},
})

export const versionNags = {
	'select-environment': {
		title: messages.environmentTitle,
		description: messages.environment,
		destination: 'versions',
	},
	'upload-version': {
		title: messages.uploadTitle,
		description: messages.upload,
		destination: 'versions',
	},
} satisfies NagDefinitions
