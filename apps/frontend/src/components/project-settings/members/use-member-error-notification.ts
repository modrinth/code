import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

const messages = defineMessages({
	title: { id: 'project.settings.members.error.title', defaultMessage: 'An error occurred' },
	unknown: { id: 'project.settings.members.error.unknown', defaultMessage: 'Unknown error' },
})

export function useMemberErrorNotification() {
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	return (error: unknown) => {
		const data = (error as { data?: { description?: unknown } } | null)?.data
		const description = typeof data?.description === 'string' ? data.description : undefined
		const message =
			error instanceof Error ? error.message : typeof error === 'string' ? error : undefined
		addNotification({
			title: formatMessage(messages.title),
			text: description || message || formatMessage(messages.unknown),
			type: 'error',
		})
	}
}
