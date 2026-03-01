import { AbstractPopupNotificationManager, type PopupNotification } from '@modrinth/ui'
import { type Ref, ref } from 'vue'

export class AppPopupNotificationManager extends AbstractPopupNotificationManager {
	private readonly state: Ref<PopupNotification[]>

	public constructor() {
		super()
		this.state = ref<PopupNotification[]>([])
	}

	public getNotifications(): PopupNotification[] {
		return this.state.value
	}

	protected addNotificationToStorage(notification: PopupNotification): void {
		this.state.value.push(notification)
	}

	protected removeNotificationFromStorage(id: string | number): void {
		const index = this.state.value.findIndex((n) => n.id === id)
		if (index > -1) {
			this.state.value.splice(index, 1)
		}
	}

	protected clearAllNotificationsFromStorage(): void {
		this.state.value.splice(0)
	}
}
