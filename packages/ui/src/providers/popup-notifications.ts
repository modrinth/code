import type { Component } from 'vue'

import { createContext } from '.'

export interface PopupNotificationButton {
	label: string
	action: () => void
	icon?: Component
	color?: 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'standard'
	keepOpen?: boolean
}

export type PopupNotificationProgressType = 'percentage' | 'bytes' | 'count'

export interface PopupNotificationProgressItem {
	id: string
	title: string
	text?: string
	iconUrl?: string | null
	progress: number
	waiting: boolean
	showProgress?: boolean
	wrapText?: boolean
	progressType?: PopupNotificationProgressType
	progressCurrent?: number
	progressTotal?: number
	onDismiss?: () => void | Promise<void>
	buttons?: PopupNotificationButton[]
}

export type PopupNotificationToastType =
	| 'friend-request'
	| 'server-invite'
	| 'instance-invite'
	| 'instance-download'
	| 'instance-ready'

export interface PopupNotificationToast {
	type: PopupNotificationToastType
	actorName?: string | null
	actorAvatarUrl?: string | null
	entityName?: string
	entityIconUrl?: string | null
	statusText?: string
	progress?: number
	waiting?: boolean
	showProgress?: boolean
	progressType?: PopupNotificationProgressType
	progressCurrent?: number
	progressTotal?: number
	onAccept?: () => void | Promise<void>
	onDecline?: () => void | Promise<void>
	onDismiss?: () => void | Promise<void>
	onLaunch?: () => void | Promise<void>
	onOpenActor?: () => void | Promise<void>
	onOpenInstance?: () => void | Promise<void>
}

export interface PopupNotification {
	id: string | number
	title: string
	titleLogo?: Component
	bodyComponent?: Component
	bodyProps?: Record<string, unknown>
	text?: string
	iconUrl?: string | null
	type?: 'error' | 'warning' | 'success' | 'info' | 'download'
	progress?: number
	waiting?: boolean
	progressItems?: PopupNotificationProgressItem[]
	buttons?: PopupNotificationButton[]
	toast?: PopupNotificationToast
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
