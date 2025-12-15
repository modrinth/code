import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonBackupsV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_backups_v0'
	}

	/** GET /modrinth/v0/servers/:server_id/backups */
	public async list(serverId: string): Promise<Archon.Backups.v1.Backup[]> {
		return this.client.request<Archon.Backups.v1.Backup[]>(`/servers/${serverId}/backups`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/** GET /modrinth/v0/servers/:server_id/backups/:backup_id */
	public async get(serverId: string, backupId: string): Promise<Archon.Backups.v1.Backup> {
		return this.client.request<Archon.Backups.v1.Backup>(
			`/servers/${serverId}/backups/${backupId}`,
			{ api: 'archon', version: 'modrinth/v0', method: 'GET' },
		)
	}

	/** POST /modrinth/v0/servers/:server_id/backups */
	public async create(
		serverId: string,
		request: Archon.Backups.v1.BackupRequest,
	): Promise<Archon.Backups.v1.PostBackupResponse> {
		return this.client.request<Archon.Backups.v1.PostBackupResponse>(
			`/servers/${serverId}/backups`,
			{ api: 'archon', version: 'modrinth/v0', method: 'POST', body: request },
		)
	}

	/** POST /modrinth/v0/servers/:server_id/backups/:backup_id/restore */
	public async restore(serverId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}/restore`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
		})
	}

	/** DELETE /modrinth/v0/servers/:server_id/backups/:backup_id */
	public async delete(serverId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'DELETE',
		})
	}

	/** POST /modrinth/v0/servers/:server_id/backups/:backup_id/lock */
	public async lock(serverId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}/lock`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
		})
	}

	/** POST /modrinth/v0/servers/:server_id/backups/:backup_id/unlock */
	public async unlock(serverId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}/unlock`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
		})
	}

	/** POST /modrinth/v0/servers/:server_id/backups/:backup_id/retry */
	public async retry(serverId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}/retry`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
		})
	}

	/** PATCH /modrinth/v0/servers/:server_id/backups/:backup_id */
	public async rename(
		serverId: string,
		backupId: string,
		request: Archon.Backups.v1.PatchBackup,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/backups/${backupId}`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'PATCH',
			body: request,
		})
	}
}
