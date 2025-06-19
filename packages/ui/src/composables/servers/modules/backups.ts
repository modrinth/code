import type { Backup, AutoBackupSettings } from "@modrinth/utils";
import { ServerModule } from "./base.js";

export class BackupsModule extends ServerModule {
  data: Backup[] = [];

  async fetch(): Promise<void> {
    this.data = await this.server.request<Backup[]>(`servers/${this.serverId}/backups`, {}, "backups");
  }

  async create(backupName: string): Promise<string> {
    const response = await this.server.request<{ id: string }>(`servers/${this.serverId}/backups`, {
      method: "POST",
      body: { name: backupName },
    });
    await this.fetch(); 
    return response.id;
  }

  async rename(backupId: string, newName: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/rename`, {
      method: "POST",
      body: { name: newName },
    });
    await this.fetch(); 
  }

  async delete(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
    await this.fetch(); 
  }

  async restore(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/restore`, {
      method: "POST",
    });
    await this.fetch(); 
  }

  async prepare(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/prepare-download`, {
      method: "POST",
    });
  }

  async lock(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/lock`, {
      method: "POST",
    });
    await this.fetch(); 
  }

  async unlock(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/unlock`, {
      method: "POST",
    });
    await this.fetch(); 
  }

  async retry(backupId: string): Promise<void> {
    await this.server.request(`servers/${this.serverId}/backups/${backupId}/retry`, {
      method: "POST",
    });
  }

  async updateAutoBackup(autoBackup: "enable" | "disable", interval: number): Promise<void> {
    await this.server.request(`servers/${this.serverId}/autobackup`, {
      method: "POST",
      body: { set: autoBackup, interval },
    });
  }

  async getAutoBackup(): Promise<AutoBackupSettings> {
    return await this.server.request(`servers/${this.serverId}/autobackup`);
  }
}
