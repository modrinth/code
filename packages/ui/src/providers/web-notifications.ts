import { createContext } from '.'

export interface WebNotification {
	id: string | number
	title?: string
	text?: string
	type?: 'error' | 'warning' | 'success' | 'info'
	errorCode?: string
	count?: number
	timer?: NodeJS.Timeout
}

export type NotificationPanelLocation = 'left' | 'right'

export abstract class AbstractWebNotificationManager {
	protected readonly AUTO_DISMISS_DELAY_MS = 30 * 1000

	abstract getNotifications(): WebNotification[]
	abstract getNotificationLocation(): NotificationPanelLocation
	abstract setNotificationLocation(location: NotificationPanelLocation): void

	protected abstract addNotificationToStorage(notification: WebNotification): void
	protected abstract removeNotificationFromStorage(id: string | number): void
	protected abstract removeNotificationFromStorageByIndex(index: number): void
	protected abstract clearAllNotificationsFromStorage(): void

	addNotification = (notification: Partial<WebNotification>): WebNotification => {
		const existingNotif = this.findExistingNotification(notification)

		if (existingNotif) {
			this.refreshNotificationTimer(existingNotif)
			existingNotif.count = (existingNotif.count || 0) + 1
			return existingNotif
		}

		const newNotification = this.createNotification(notification)
		this.setNotificationTimer(newNotification)
		this.addNotificationToStorage(newNotification)
		return newNotification
	}

	/**
	 * @deprecated You should use `addNotification` instead to provide a more human-readable error message to the user.
	 */
	handleError = (error: Error): void => {
		this.addNotification({
			title: 'An error occurred',
			text: error.message ?? error,
			type: 'error',
		})
	}

	removeNotification = (id: string | number): WebNotification | undefined => {
		const notifications = this.getNotifications()
		const notification = notifications.find((n) => n.id === id)

		if (notification) {
			this.clearNotificationTimer(notification)
			this.removeNotificationFromStorage(id)
		}

		return notification
	}

	removeNotificationByIndex = (index: number): WebNotification | null => {
		const notifications = this.getNotifications()

		if (index >= 0 && index < notifications.length) {
			const notification = notifications[index]
			this.clearNotificationTimer(notification)
			this.removeNotificationFromStorageByIndex(index)

			return notification
		}

		return null
	}

	clearAllNotifications = (): void => {
		const notifications = this.getNotifications()
		notifications.forEach((notification) => {
			this.clearNotificationTimer(notification)
		})
		this.clearAllNotificationsFromStorage()
	}

	setNotificationTimer = (notification: WebNotification): void => {
		if (!notification) return

		this.clearNotificationTimer(notification)

		notification.timer = setTimeout(() => {
			this.removeNotification(notification.id)
		}, this.AUTO_DISMISS_DELAY_MS)
	}

	stopNotificationTimer = (notification: WebNotification): void => {
		this.clearNotificationTimer(notification)
	}

	private refreshNotificationTimer(notification: WebNotification): void {
		this.setNotificationTimer(notification)
	}

	private clearNotificationTimer(notification: WebNotification): void {
		if (notification.timer) {
			clearTimeout(notification.timer)
			notification.timer = undefined
		}
	}

	private findExistingNotification(
		notification: Partial<WebNotification>,
	): WebNotification | undefined {
		return this.getNotifications().find(
			(existing) =>
				existing.text === notification.text &&
				existing.title === notification.title &&
				existing.type === notification.type,
		)
	}

	private createNotification(notification: Partial<WebNotification>): WebNotification {
		return {
			...notification,
			id: new Date().getTime(),
			count: 1,
		} as WebNotification
	}
}

export const [injectNotificationManager, provideNotificationManager] =
	createContext<AbstractWebNotificationManager>('root', 'notificationManager')
