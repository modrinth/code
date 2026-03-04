import { createContext } from '.'

export interface PopupNotificationButton {
	label: string
	action: () => void
	color?: 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'standard'
}

export interface PopupNotification {
	id: string | number
	title: string
	text?: string
	type?: 'error' | 'warning' | 'success' | 'info'
	buttons?: PopupNotificationButton[]
	autoCloseMs?: number | null
	timer?: NodeJS.Timeout
}

export abstract class AbstractPopupNotificationManager {
	protected readonly DEFAULT_AUTO_CLOSE_MS = 30 * 1000

	abstract getNotifications(): PopupNotification[]

	protected abstract addNotificationToStorage(notification: PopupNotification): void
	protected abstract removeNotificationFromStorage(id: string | number): void
	protected abstract clearAllNotificationsFromStorage(): void

	addPopupNotification = (
		notification: Omit<PopupNotification, 'id' | 'timer'>,
	): PopupNotification => {
		const newNotification: PopupNotification = {
			...notification,
			id: Date.now() + Math.random(),
		}
		this.setNotificationTimer(newNotification)
		this.addNotificationToStorage(newNotification)
		return newNotification
	}

	removeNotification = (id: string | number): void => {
		const notifications = this.getNotifications()
		const notification = notifications.find((n) => n.id === id)
		if (notification) {
			this.clearNotificationTimer(notification)
			this.removeNotificationFromStorage(id)
		}
	}

	clearAllNotifications = (): void => {
		this.getNotifications().forEach((n) => this.clearNotificationTimer(n))
		this.clearAllNotificationsFromStorage()
	}

	setNotificationTimer = (notification: PopupNotification): void => {
		if (!notification) return
		this.clearNotificationTimer(notification)

		if (notification.autoCloseMs === null) return

		const delay = notification.autoCloseMs ?? this.DEFAULT_AUTO_CLOSE_MS
		notification.timer = setTimeout(() => {
			this.removeNotification(notification.id)
		}, delay)
	}

	stopNotificationTimer = (notification: PopupNotification): void => {
		this.clearNotificationTimer(notification)
	}

	private clearNotificationTimer(notification: PopupNotification): void {
		if (notification.timer) {
			clearTimeout(notification.timer)
			notification.timer = undefined
		}
	}
}

export const [injectPopupNotificationManager, providePopupNotificationManager] =
	createContext<AbstractPopupNotificationManager>('root', 'popupNotificationManager')
