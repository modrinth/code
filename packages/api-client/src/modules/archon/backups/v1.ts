import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonBackupsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_backups_v1'
	}

	/** GET /v1/servers/:server_id/worlds/:world_id/backups */
	public async list(serverId: string, worldId: string): Promise<Archon.Backups.v1.Backup[]> {
		return this.client.request<Archon.Backups.v1.Backup[]>(
			`/servers/${serverId}/worlds/${worldId}/backups`,
			{ api: 'archon', version: 1, method: 'GET' },
		)
	}

	/** GET /v1/servers/:server_id/worlds/:world_id/backups/:backup_id */
	public async get(
		serverId: string,
		worldId: string,
		backupId: string,
	): Promise<Archon.Backups.v1.Backup> {
		return this.client.request<Archon.Backups.v1.Backup>(
			`/servers/${serverId}/worlds/${worldId}/backups/${backupId}`,
			{ api: 'archon', version: 1, method: 'GET' },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups */
	public async create(
		serverId: string,
		worldId: string,
		request: Archon.Backups.v1.BackupRequest,
	): Promise<Archon.Backups.v1.PostBackupResponse> {
		return this.client.request<Archon.Backups.v1.PostBackupResponse>(
			`/servers/${serverId}/worlds/${worldId}/backups`,
			{ api: 'archon', version: 1, method: 'POST', body: request },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups/:backup_id/restore */
	public async restore(serverId: string, worldId: string, backupId: string): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups/${backupId}/restore`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
			},
		)
	}

	/** DELETE /v1/servers/:server_id/worlds/:world_id/backups/:backup_id */
	public async delete(serverId: string, worldId: string, backupId: string): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/backups/${backupId}`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups/:backup_id/retry */
	public async retry(serverId: string, worldId: string, backupId: string): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups/${backupId}/retry`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
			},
		)
	}

	/** PATCH /v1/servers/:server_id/worlds/:world_id/backups/:backup_id */
	public async rename(
		serverId: string,
		worldId: string,
		backupId: string,
		request: Archon.Backups.v1.PatchBackup,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/backups/${backupId}`, {
			api: 'archon',
			version: 1,
			method: 'PATCH',
			body: request,
		})
	}
}
