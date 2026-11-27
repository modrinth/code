import type { AbstractWebNotificationManager, WebNotification } from '@modrinth/ui'

export function addUndoNotification(
	manager: AbstractWebNotificationManager,
	notification: Partial<WebNotification> & { id: string; title: string },
) {
	const matchingTitles = new Set(
		manager
			.getNotifications()
			.filter((item) => item.type === 'success' && item.text === notification.text)
			.map((item) => item.title),
	)
	let title = notification.title
	for (let number = 2; matchingTitles.has(title); number++) {
		title = `${notification.title} (${number})`
	}
	manager.addNotification({ ...notification, type: 'success', title })
}
