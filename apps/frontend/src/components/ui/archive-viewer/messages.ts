import { defineMessages } from '@modrinth/ui'

export const channelMessages = defineMessages({
	release: { id: 'archive.project-details.channel.release', defaultMessage: 'Release' },
	beta: { id: 'archive.project-details.channel.beta', defaultMessage: 'Beta' },
	alpha: { id: 'archive.project-details.channel.alpha', defaultMessage: 'Alpha' },
})

export const versionStatusMessages = defineMessages({
	listed: { id: 'archive.project-details.version-status.listed', defaultMessage: 'Listed' },
	archived: { id: 'archive.project-details.version-status.archived', defaultMessage: 'Archived' },
	draft: { id: 'archive.project-details.version-status.draft', defaultMessage: 'Draft' },
	unlisted: { id: 'archive.project-details.version-status.unlisted', defaultMessage: 'Unlisted' },
	scheduled: {
		id: 'archive.project-details.version-status.scheduled',
		defaultMessage: 'Scheduled',
	},
	unknown: { id: 'archive.project-details.version-status.unknown', defaultMessage: 'Unknown' },
})
