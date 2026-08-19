import { type Component, markRaw, type Ref, ref } from 'vue'
import type { ComponentProps } from 'vue-component-type-helpers'

import type { ButtonColor } from '#ui/components/base/buttons/types.ts'

import { createContext } from '.'

export interface PopupNotificationButton {
	label: string
	action: () => void | Promise<void>
	icon?: Component
	color?: ButtonColor | 'standard'
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

interface PopupNotificationBase {
	id: string | number
	autoCloseMs?: number | null
	timer?: NodeJS.Timeout
}

export interface PopupNotificationCustom<
	TComponent extends Component = Component,
> extends PopupNotificationBase {
	contentType: 'custom'
	component: TComponent
	componentProps?: ComponentProps<TComponent>
}

export interface PopupNotificationStandard extends PopupNotificationBase {
	contentType: 'standard'
	title: string
	type: 'error' | 'warning' | 'success' | 'info' | 'download'
	titleLogo?: Component
	text?: string
	iconUrl?: string | null
	hideIcon?: boolean
	progress?: number
	waiting?: boolean
	progressItems?: PopupNotificationProgressItem[]
	buttons?: PopupNotificationButton[]
	dismissible?: boolean
	onDismiss?: () => void | Promise<void>
}

export interface PopupNotificationToast extends PopupNotificationBase {
	contentType: 'toast'
	title: string
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

export type PopupNotification =
	| PopupNotificationCustom
	| PopupNotificationStandard
	| PopupNotificationToast

type PopupNotificationDownload =
	| (PopupNotificationStandard & { type: 'download' })
	| (PopupNotificationToast & { type: 'instance-download' })

type PopupNotificationInput = PopupNotification extends infer Notification
	? Notification extends PopupNotification
		? Omit<Notification, 'id' | 'timer'>
		: never
	: never

type PopupNotificationCustomInput<TComponent extends Component> = Omit<
	PopupNotificationCustom<TComponent>,
	'id' | 'timer'
>

type PopupNotificationNonCustomInput =
	| Omit<PopupNotificationStandard, 'id' | 'timer'>
	| Omit<PopupNotificationToast, 'id' | 'timer'>

type StoredPopupNotification<Input extends PopupNotificationInput> = Input &
	Pick<PopupNotificationBase, 'id' | 'timer'>

interface AddPopupNotification {
	<TComponent extends Component>(
		notification: PopupNotificationCustomInput<TComponent>,
	): StoredPopupNotification<PopupNotificationCustomInput<TComponent>>
	<Input extends PopupNotificationNonCustomInput>(
		notification: Input,
	): StoredPopupNotification<Input>
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
		notification: PopupNotificationStandard,
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

	addPopupNotification = ((notification: PopupNotificationInput) => {
		const newNotification = {
			...notification,
			...(notification.contentType === 'custom'
				? { component: markRaw(notification.component) }
				: {}),
			id: Date.now() + Math.random(),
		} as StoredPopupNotification<PopupNotificationInput>
		this.setNotificationTimer(newNotification)
		this.addNotificationToStorage(newNotification)
		return newNotification
	}) as AddPopupNotification

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

	hideDownloadItem = (notificationId: string | number, progressItemId?: string): void => {
		const notification = this.getDownloadNotifications().find(
			(candidate) => candidate.id === notificationId,
		)
		if (!notification) return

		const itemKey = this.getDownloadItemKey(notificationId, progressItemId)
		const itemKeys = this.getDownloadItemKeys(notification)
		if (!itemKeys.includes(itemKey)) return

		this.hiddenDownloadItemKeys.value.add(itemKey)
		if (itemKeys.every((key) => this.hiddenDownloadItemKeys.value.has(key))) {
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

	private isDownloadNotification(
		notification: PopupNotification,
	): notification is PopupNotificationDownload {
		return (
			(notification.contentType === 'standard' && notification.type === 'download') ||
			(notification.contentType === 'toast' && notification.type === 'instance-download')
		)
	}

	private getDownloadNotifications(): PopupNotificationDownload[] {
		return this.getNotifications().filter((notification) =>
			this.isDownloadNotification(notification),
		)
	}

	private getDownloadItemKeys(notification: PopupNotification): string[] {
		if (!this.isDownloadNotification(notification)) return []
		if (notification.contentType === 'standard' && notification.progressItems?.length) {
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
