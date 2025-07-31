import { useState } from "#app";
import {
  type WebNotification,
  type WebNotificationLocation,
  AbstractWebNotificationManager,
} from "@modrinth/ui";

export class FrontendNotificationManager extends AbstractWebNotificationManager {
  private readonly state: Ref<WebNotification[]>;
  private readonly locationState: Ref<WebNotificationLocation>;

  public constructor() {
    super();
    this.state = useState<WebNotification[]>("notifications", () => []);
    this.locationState = useState<WebNotificationLocation>("notifications.location", () => "right");
  }

  public getNotificationLocation(): WebNotificationLocation {
    return this.locationState.value;
  }

  public setNotificationLocation(location: WebNotificationLocation): void {
    this.locationState.value = location;
  }

  public getNotifications(): WebNotification[] {
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
