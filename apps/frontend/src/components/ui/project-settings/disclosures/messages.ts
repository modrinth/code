import { defineMessages } from '@modrinth/ui'

export const disclosureStatusMessages = defineMessages({
	loading: { id: 'project.disclosures.loading', defaultMessage: 'Loading disclosures…' },
	loadError: {
		id: 'project.disclosures.load-error',
		defaultMessage: 'Could not load disclosures.',
	},
	retry: { id: 'project.disclosures.retry', defaultMessage: 'Retry' },
	saveError: {
		id: 'project.disclosures.save-error',
		defaultMessage:
			'Could not save all disclosure changes. Your edits have been kept. Try saving again.',
	},
})
