import { defineMessages } from '@modrinth/ui'

export const licenseUrlMessages = defineMessages({
	url: { id: 'project.settings.license.url', defaultMessage: 'License URL' },
	optionalUrl: {
		id: 'project.settings.license.optional-url',
		defaultMessage: 'License URL (optional)',
	},
	urlDescription: {
		id: 'project.settings.license.url-description',
		defaultMessage:
			"The web location of the full license text. If you don't provide a link, the license text will be displayed instead.",
	},
	customUrlDescription: {
		id: 'project.settings.license.custom-url-description',
		defaultMessage:
			'The web location of the full license text. You have to provide a link since this is a custom license.',
	},
})
