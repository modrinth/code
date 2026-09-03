import { defineMessages } from '@modrinth/ui/i18n'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	title: { id: 'nags.invalid-gallery-text.title', defaultMessage: 'Modify gallery image text' },
	nonStandard: {
		id: 'nags.gallery-text-non-standard.description',
		defaultMessage:
			'Non-standard text characters, such as “Fancy text” or “Zalgo”, are not allowed in gallery image titles or descriptions.',
	},
	profanity: {
		id: 'nags.gallery-text-profanity.description',
		defaultMessage:
			'Your gallery image titles and descriptions cannot contain excessive profanity. Detected: “{value}”.',
	},
	slur: {
		id: 'nags.gallery-text-slur.description',
		defaultMessage:
			'Your gallery image titles and descriptions must not contain offensive terms. Detected: “{value}”.',
	},
	editGallery: { id: 'nags.edit-gallery.title', defaultMessage: 'Edit gallery' },
})

export const galleryTextNags = {
	'gallery-text-non-standard': {
		title: messages.title,
		description: messages.nonStandard,
		destination: 'gallery',
		linkTitle: messages.editGallery,
	},
	'gallery-text-profanity': {
		title: messages.title,
		description: messages.profanity,
		destination: 'gallery',
		linkTitle: messages.editGallery,
	},
	'gallery-text-slur': {
		title: messages.title,
		description: messages.slur,
		destination: 'gallery',
		linkTitle: messages.editGallery,
	},
} satisfies NagDefinitions
