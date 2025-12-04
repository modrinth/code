import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

/**
 * Default world ID - Uuid::nil() which the backend treats as "first/active world"
 * See: apps/archon/src/routes/v1/servers/worlds/mod.rs - world_id_nullish()
 * TODO:
 * - Make sure world ID is being passed before we ship worlds.
 * - The schema will change when Backups v4 (routes stay as v1) so remember to do that.
 */
const DEFAULT_WORLD_ID: string = '00000000-0000-0000-0000-000000000000' as const

export class ArchonBackupsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_backups_v1'
	}

	/** GET /v1/:server_id/worlds/:world_id/backups */
	public async list(
		serverId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<Archon.Backups.v1.Backup[]> {
		return this.client.request<Archon.Backups.v1.Backup[]>(
			`/${serverId}/worlds/${worldId}/backups`,
			{ api: 'archon', version: 1, method: 'GET' },
		)
	}

	/** GET /v1/:server_id/worlds/:world_id/backups/:backup_id */
	public async get(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<Archon.Backups.v1.Backup> {
		return this.client.request<Archon.Backups.v1.Backup>(
			`/${serverId}/worlds/${worldId}/backups/${backupId}`,
			{ api: 'archon', version: 1, method: 'GET' },
		)
	}

	/** POST /v1/:server_id/worlds/:world_id/backups */
	public async create(
		serverId: string,
		request: Archon.Backups.v1.BackupRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<Archon.Backups.v1.PostBackupResponse> {
		return this.client.request<Archon.Backups.v1.PostBackupResponse>(
			`/${serverId}/worlds/${worldId}/backups`,
			{ api: 'archon', version: 1, method: 'POST', body: request },
		)
	}

	/** POST /v1/:server_id/worlds/:world_id/backups/:backup_id/restore */
	public async restore(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}/restore`, {
			api: 'archon',
			version: 1,
			method: 'POST',
		})
	}

	/** DELETE /v1/:server_id/worlds/:world_id/backups/:backup_id */
	public async delete(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/backups/:backup_id/lock */
	public async lock(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}/lock`, {
			api: 'archon',
			version: 1,
			method: 'POST',
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/backups/:backup_id/unlock */
	public async unlock(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}/unlock`, {
			api: 'archon',
			version: 1,
			method: 'POST',
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/backups/:backup_id/retry */
	public async retry(
		serverId: string,
		backupId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}/retry`, {
			api: 'archon',
			version: 1,
			method: 'POST',
		})
	}

	/** PATCH /v1/:server_id/worlds/:world_id/backups/:backup_id */
	public async rename(
		serverId: string,
		backupId: string,
		request: Archon.Backups.v1.PatchBackup,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/${serverId}/worlds/${worldId}/backups/${backupId}`, {
			api: 'archon',
			version: 1,
			method: 'PATCH',
			body: request,
		})
	}
}
