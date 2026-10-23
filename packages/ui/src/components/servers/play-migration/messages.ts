import { defineMessages } from '#ui/composables/i18n'

export const serverPlayMigrationMessages = defineMessages({
	title: {
		id: 'servers.play-migration.title',
		defaultMessage: 'Get your server ready',
	},
	backupName: {
		id: 'servers.play-migration.backup.name',
		defaultMessage: 'Before reinstalling content',
	},
	inProgress: {
		id: 'servers.play-migration.in-progress',
		defaultMessage: 'Getting your server ready to share with friends. This may take a few minutes.',
	},
	backingUp: {
		id: 'servers.play-migration.step.backup',
		defaultMessage: 'Backing up your server',
	},
	reinstalling: {
		id: 'servers.play-migration.step.reinstall',
		defaultMessage: 'Reinstalling your content',
	},
	restarting: {
		id: 'servers.play-migration.step.restart',
		defaultMessage: 'Restarting your server',
	},
	contentLocked: {
		id: 'servers.play-migration.content-locked',
		defaultMessage: 'Get your server ready from the Play tab to choose where content is enabled.',
	},
})
