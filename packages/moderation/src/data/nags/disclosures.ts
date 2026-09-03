import { defineMessages } from '@modrinth/ui/i18n'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	checkTitle: { id: 'nags.check-disclosures.title', defaultMessage: 'Review disclosures' },
	check: {
		id: 'nags.check-disclosures.description',
		defaultMessage:
			'Make sure users are aware of any important details by filling in content disclosures that apply to your {type}.',
	},
	formattingTitle: {
		id: 'nags.disclosures-special-formatting.title',
		defaultMessage: 'Fix disclosure formatting',
	},
	formatting: {
		id: 'nags.disclosures-special-formatting.description',
		defaultMessage:
			'Content disclosures should not contain HTML, since they can only display inline Markdown and plain text.',
	},
})

export const disclosureNags = {
	'check-disclosures': {
		title: messages.checkTitle,
		description: messages.check,
		destination: 'disclosures',
	},
	'disclosures-special-formatting': {
		title: messages.formattingTitle,
		description: messages.formatting,
		destination: 'disclosures',
	},
} satisfies NagDefinitions
