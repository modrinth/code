import { defineMessages } from '#ui/composables/i18n'

export const fileValidationMessages = defineMessages({
	nameLabel: {
		id: 'files.validation.name-label',
		defaultMessage: 'Name',
	},
	nameRequired: {
		id: 'files.validation.name-required',
		defaultMessage: 'Name is required.',
	},
	nameInvalidFile: {
		id: 'files.validation.name-invalid-file',
		defaultMessage:
			'Name must contain only alphanumeric characters, dashes, underscores, dots, or spaces.',
	},
	nameInvalidDirectory: {
		id: 'files.validation.name-invalid-directory',
		defaultMessage:
			'Name must contain only alphanumeric characters, dashes, underscores, or spaces.',
	},
})
