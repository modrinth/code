import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

const messages = defineMessages({
	copiedTitle: {
		id: 'app.settings.resource-management.app-directory.path-copied-title',
		defaultMessage: 'Path copied',
	},
	copiedText: {
		id: 'app.settings.resource-management.app-directory.path-copied-text',
		defaultMessage: 'The app directory path has been copied to your clipboard.',
	},
	copyFailedTitle: {
		id: 'app.settings.resource-management.app-directory.path-copy-failed-title',
		defaultMessage: 'Failed to copy path',
	},
})

export function useCopyPath() {
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()

	return async function copyPath(path: string) {
		try {
			await navigator.clipboard.writeText(path)
			addNotification({
				type: 'success',
				title: formatMessage(messages.copiedTitle),
				text: formatMessage(messages.copiedText),
			})
		} catch (error) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.copyFailedTitle),
				text: error instanceof Error ? error.message : String(error),
			})
		}
	}
}
