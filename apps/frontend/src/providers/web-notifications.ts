import { type WebNotification, AbstractWebNotificationManager } from "@modrinth/ui";
import { useState } from "#app";

export class FrontendNotificationManager extends AbstractWebNotificationManager {
  private state: globalThis.Ref<WebNotification[], WebNotification[]>;

  constructor() {
    super();
    this.state = useState<WebNotification[]>("notifications", () => []);
  }

  getNotifications(): WebNotification[] {
    return this.state.value;
  }

  protected addNotificationToStorage(notification: WebNotification): void {
    this.state.value.push(notification);
  }

  protected removeNotificationFromStorage(id: string | number): void {
    const index = this.state.value.findIndex((n) => n.id === id);
    if (index > -1) {
      this.state.value.splice(index, 1);
    }
  }

  protected removeNotificationFromStorageByIndex(index: number): void {
    this.state.value.splice(index, 1);
  }

  protected clearAllNotificationsFromStorage(): void {
    this.state.value.splice(0);
  }
}
