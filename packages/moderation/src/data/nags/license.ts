import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	detailsTitle: {
		id: 'nags.add-license-details.title',
		defaultMessage: 'Add license details',
	},
	details: {
		id: 'nags.add-license-details.description',
		defaultMessage: 'Add a valid URL and name or SPDX identifier for your custom license.',
	},
	urlTitle: { id: 'nags.invalid-license-url.title', defaultMessage: 'Add a valid license link' },
	url: {
		id: 'nags.invalid-license-url.description.default',
		defaultMessage: 'License URL is invalid.',
	},
	urlDomain: {
		id: 'nags.invalid-license-url.description.domain',
		defaultMessage:
			'Your license URL points to {domain}, which is not appropriate for license information. License URLs should link directly to your license text.',
	},
	urlMalformed: {
		id: 'nags.invalid-license-url.description.malformed',
		defaultMessage:
			'Your license URL appears to be malformed. Please provide a valid URL to your license text.',
	},
	selectTitle: { id: 'nags.select-license.title', defaultMessage: 'Select a license' },
	select: {
		id: 'nags.select-license.description',
		defaultMessage: 'Select the license your {type} is distributed under.',
	},
	editLicense: { id: 'nags.edit-license.title', defaultMessage: 'Edit license' },
})

export const licenseNags = {
	'add-custom-license-details': {
		title: messages.detailsTitle,
		description: messages.details,
		destination: 'license',
	},
	'invalid-license-url': {
		title: messages.urlTitle,
		description: ({ nag }) => {
			if (typeof nag.details.domain === 'string') return messages.urlDomain
			if (nag.details.reason === 'malformed') return messages.urlMalformed
			return messages.url
		},
		destination: 'license',
		linkTitle: messages.editLicense,
	},
	'select-license': {
		title: messages.selectTitle,
		description: messages.select,
		destination: 'license',
	},
} satisfies NagDefinitions
