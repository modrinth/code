import { type Component, type Ref, ref } from 'vue'

import { createContext } from '.'

export interface PopupNotificationButton {
	label: string
	action: () => void | Promise<void>
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
	dismissible?: boolean
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
	hideIcon?: boolean
	type?: 'error' | 'warning' | 'success' | 'info' | 'download'
	progress?: number
	waiting?: boolean
	progressItems?: PopupNotificationProgressItem[]
	buttons?: PopupNotificationButton[]
	toast?: PopupNotificationToast
	dismissible?: boolean
	onDismiss?: () => void | Promise<void>
	autoCloseMs?: number | null
	timer?: NodeJS.Timeout
}

export interface PopupNotificationDownloadState {
	total: number
	hidden: number
}

export abstract class AbstractPopupNotificationManager {
	protected readonly DEFAULT_AUTO_CLOSE_MS = 30 * 1000
	private readonly hiddenDownloadItemKeys: Ref<Set<string>> = ref(new Set())

	abstract getNotifications(): PopupNotification[]

	getDownloadState = (): PopupNotificationDownloadState => {
		const itemKeys = this.getDownloadNotifications().flatMap((notification) =>
			this.getDownloadItemKeys(notification),
		)
		return {
			total: itemKeys.length,
			hidden: itemKeys.filter((key) => this.hiddenDownloadItemKeys.value.has(key)).length,
		}
	}

	getVisibleNotifications = (): PopupNotification[] =>
		this.getNotifications().filter(
			(notification) =>
				!this.isDownloadNotification(notification) ||
				this.getDownloadItemKeys(notification).some(
					(key) => !this.hiddenDownloadItemKeys.value.has(key),
				),
		)

	getVisibleDownloadProgressItems = (
		notification: PopupNotification,
	): PopupNotificationProgressItem[] =>
		(notification.progressItems ?? []).filter(
			(progressItem) =>
				!this.hiddenDownloadItemKeys.value.has(
					this.getDownloadItemKey(notification.id, progressItem.id),
				),
		)

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
			this.getDownloadItemKeys(notification).forEach((key) =>
				this.hiddenDownloadItemKeys.value.delete(key),
			)
			this.removeNotificationFromStorage(id)
		}
	}

	clearAllNotifications = (): void => {
		this.getNotifications().forEach((n) => this.clearNotificationTimer(n))
		this.clearAllNotificationsFromStorage()
		this.hiddenDownloadItemKeys.value.clear()
	}

	hideDownloadItem = (notificationId: string | number, progressItemId: string): void => {
		const notification = this.getDownloadNotifications().find(
			(candidate) => candidate.id === notificationId,
		)
		if (!notification?.progressItems?.some((item) => item.id === progressItemId)) return

		this.hiddenDownloadItemKeys.value.add(this.getDownloadItemKey(notificationId, progressItemId))
		if (
			this.getDownloadItemKeys(notification).every((key) =>
				this.hiddenDownloadItemKeys.value.has(key),
			)
		) {
			this.clearNotificationTimer(notification)
		}
	}

	toggleDownloadNotifications = (): void => {
		const downloadNotifications = this.getDownloadNotifications()
		const hasHiddenDownloads = downloadNotifications.some((notification) =>
			this.getDownloadItemKeys(notification).some((key) =>
				this.hiddenDownloadItemKeys.value.has(key),
			),
		)

		if (hasHiddenDownloads) {
			this.hiddenDownloadItemKeys.value.clear()
			downloadNotifications.forEach((notification) => this.setNotificationTimer(notification))
			return
		}

		downloadNotifications.forEach((notification) => {
			this.getDownloadItemKeys(notification).forEach((key) =>
				this.hiddenDownloadItemKeys.value.add(key),
			)
			this.clearNotificationTimer(notification)
		})
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

	private isDownloadNotification(notification: PopupNotification): boolean {
		return notification.type === 'download' || notification.toast?.type === 'instance-download'
	}

	private getDownloadNotifications(): PopupNotification[] {
		return this.getNotifications().filter((notification) =>
			this.isDownloadNotification(notification),
		)
	}

	private getDownloadItemKeys(notification: PopupNotification): string[] {
		if (!this.isDownloadNotification(notification)) return []
		if (notification.progressItems?.length) {
			return notification.progressItems.map((progressItem) =>
				this.getDownloadItemKey(notification.id, progressItem.id),
			)
		}
		return [this.getDownloadItemKey(notification.id)]
	}

	private getDownloadItemKey(notificationId: string | number, progressItemId?: string): string {
		return JSON.stringify([typeof notificationId, notificationId, progressItemId ?? null])
	}
}

export const [injectPopupNotificationManager, providePopupNotificationManager] =
	createContext<AbstractPopupNotificationManager>('root', 'popupNotificationManager')
