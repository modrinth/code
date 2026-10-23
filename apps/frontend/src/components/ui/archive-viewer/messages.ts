import { defineMessages } from '@modrinth/ui'

/** Links to the archive viewer from elsewhere, e.g. version lists. */
export const archiveLinkMessages = defineMessages({
	archive: { id: 'archive.link.archive', defaultMessage: 'Archive' },
	openArchive: { id: 'archive.link.open', defaultMessage: 'Open in the archive viewer' },
})

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
