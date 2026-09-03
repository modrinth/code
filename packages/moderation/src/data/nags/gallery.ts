import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	featureTitle: {
		id: 'nags.feature-gallery-image.title',
		defaultMessage: 'Feature a gallery image',
	},
	feature: {
		id: 'nags.feature-gallery-image.description',
		defaultMessage:
			'The featured gallery image is often how your project makes its first impression.',
	},
	uploadTitle: { id: 'nags.upload-gallery-image.title', defaultMessage: 'Upload a gallery image' },
	upload: {
		id: 'nags.upload-gallery-image.description',
		defaultMessage:
			'At least one gallery image is required to showcase the content of your {type}.',
	},
	uploadResourcePack: {
		id: 'nags.upload-gallery-image.description-resourcepack',
		defaultMessage:
			'At least one gallery image is required to showcase the content of your resource pack, except for audio or localization packs. If this describes your pack, please select the appropriate tag.',
	},
	uploadShader: {
		id: 'nags.upload-gallery-image.description-shader',
		defaultMessage:
			'At least three gallery images are required to showcase the content of your shader in a variety of situations and conditions.',
	},
})

export const galleryNags = {
	'feature-gallery-image': {
		title: messages.featureTitle,
		description: messages.feature,
		destination: 'gallery',
	},
	'upload-gallery-image': {
		title: messages.uploadTitle,
		description: ({ projectType }) => {
			if (projectType === 'resourcepack') return messages.uploadResourcePack
			if (projectType === 'shader') return messages.uploadShader
			return messages.upload
		},
		destination: 'gallery',
	},
} satisfies NagDefinitions
