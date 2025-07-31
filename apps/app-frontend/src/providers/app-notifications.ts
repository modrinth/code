import {
  AbstractWebNotificationManager,
  type WebNotification,
  type WebNotificationLocation,
} from '@modrinth/ui'
import { ref, type Ref } from 'vue'

export class AppNotificationManager extends AbstractWebNotificationManager {
  private readonly state: Ref<WebNotification[]>
  private readonly locationState: Ref<WebNotificationLocation>

  public constructor() {
    super()
    this.state = ref<WebNotification[]>([])
    this.locationState = ref<WebNotificationLocation>('right')
  }

  public handleError = (error: Error): void => {
    this.addNotification({
      title: 'An error occurred',
      text: error.message ?? error,
      type: 'error',
    })
  }

  public getNotificationLocation(): WebNotificationLocation {
    return this.locationState.value
  }

  public setNotificationLocation(location: WebNotificationLocation): void {
    this.locationState.value = location
  }

  public getNotifications(): WebNotification[] {
    return this.state.value
  }

  protected addNotificationToStorage(notification: WebNotification): void {
    this.state.value.push(notification)
  }

  protected removeNotificationFromStorage(id: string | number): void {
    const index = this.state.value.findIndex((n) => n.id === id)
    if (index > -1) {
      this.state.value.splice(index, 1)
    }
  }

  protected removeNotificationFromStorageByIndex(index: number): void {
    this.state.value.splice(index, 1)
  }

  protected clearAllNotificationsFromStorage(): void {
    this.state.value.splice(0)
  }
}
