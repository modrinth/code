import { defineMessages } from '#ui/composables/i18n'

export const serverPlayMigrationMessages = defineMessages({
	title: {
		id: 'servers.play-migration.title',
		defaultMessage: 'Get your server ready',
	},
	backupName: {
		id: 'servers.play-migration.backup.name',
		defaultMessage: 'Before reinstalling server content',
	},
	contentLocked: {
		id: 'servers.play-migration.content-locked',
		defaultMessage: 'Share your server on the Play tab to choose where content is enabled.',
	},
	confirm: {
		id: 'servers.play-migration.confirm',
		defaultMessage: 'Backup & reinstall',
	},
})
