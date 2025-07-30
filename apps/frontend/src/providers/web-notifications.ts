import { type WebNotification, AbstractWebNotificationManager } from "@modrinth/ui";
import { useState } from "#app";

export class FrontendNotificationManager extends AbstractWebNotificationManager {
  private readonly state: Ref<WebNotification[]>;
  private readonly isRightwards: Ref<boolean>;

  public constructor() {
    super();
    this.state = useState<WebNotification[]>("notifications", () => []);
    this.isRightwards = useState<boolean>("notifications.isRightwards", () => true);
  }

  public getNotifications(): WebNotification[] {
    return this.state.value;
  }

  public isNotificationsPanelRightwards(): boolean {
    return this.isRightwards.value;
  }

  public setNotificationsPanelRightwards(isRightwards: boolean): void {
    this.isRightwards.value = isRightwards;
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
