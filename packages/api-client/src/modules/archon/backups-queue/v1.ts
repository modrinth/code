import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonBackupsQueueV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_backups_queue_v1'
	}

	/** GET /v1/servers/:server_id/worlds/:world_id/backups-queue */
	public async list(
		serverId: string,
		worldId: string,
	): Promise<Archon.BackupsQueue.v1.BackupsQueueResponse> {
		return this.client.request<Archon.BackupsQueue.v1.BackupsQueueResponse>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue`,
			{ api: 'archon', version: 1, method: 'GET' },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue */
	public async create(
		serverId: string,
		worldId: string,
		request: Archon.BackupsQueue.v1.BackupRequest,
	): Promise<Archon.BackupsQueue.v1.PostBackupQueueResponse> {
		return this.client.request<Archon.BackupsQueue.v1.PostBackupQueueResponse>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue`,
			{ api: 'archon', version: 1, method: 'POST', body: request },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue/history/create/:operation_id/ack */
	public async ackCreate(serverId: string, worldId: string, operationId: number): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/history/create/${operationId}/ack`,
			{ api: 'archon', version: 1, method: 'POST' },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue/history/restore/:operation_id/ack */
	public async ackRestore(serverId: string, worldId: string, operationId: number): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/history/restore/${operationId}/ack`,
			{ api: 'archon', version: 1, method: 'POST' },
		)
	}

	/** DELETE /v1/servers/:server_id/worlds/:world_id/backups-queue/:backup_id */
	public async delete(serverId: string, worldId: string, backupId: string): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/${backupId}`,
			{
				api: 'archon',
				version: 1,
				method: 'DELETE',
			},
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue/delete-many */
	public async deleteMany(serverId: string, worldId: string, backupIds: string[]): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/delete-many`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
				body: { backup_ids: backupIds } satisfies Archon.BackupsQueue.v1.DeleteManyBackupRequest,
			},
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue/:backup_id/restore */
	public async restore(
		serverId: string,
		worldId: string,
		backupId: string,
		request: Archon.BackupsQueue.v1.BackupRequest,
	): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/${backupId}/restore`,
			{ api: 'archon', version: 1, method: 'POST', body: request },
		)
	}

	/** POST /v1/servers/:server_id/worlds/:world_id/backups-queue/:backup_id/retry */
	public async retry(serverId: string, worldId: string, backupId: string): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/backups-queue/${backupId}/retry`,
			{ api: 'archon', version: 1, method: 'POST' },
		)
	}
}
