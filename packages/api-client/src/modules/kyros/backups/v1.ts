import { AbstractModule } from '../../../core/abstract-module'
import { getNodeBaseUrl } from '../../../utils/node-url'
import type { Kyros } from '../types'

export class KyrosBackupsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_backups_v1'
	}

	/**
	 * Authorize a short-lived, single-use backup download.
	 */
	public async authorizeDownload(
		nodeUrlHost: string,
		worldId: string,
		backupId: string,
		format?: Kyros.Backups.v1.BackupDumpFormat,
	): Promise<Kyros.Backups.v1.BackupDownloadAuthorization> {
		return this.client.request<Kyros.Backups.v1.BackupDownloadAuthorization>(
			`/worlds/${worldId}/backups/${backupId}/download/authorize`,
			{
				api: getNodeBaseUrl(nodeUrlHost),
				version: 'v1',
				method: 'GET',
				params: format ? { format } : undefined,
			},
		)
	}

	/**
	 * Build the browser URL for a previously authorized backup download.
	 */
	public getDownloadUrl(
		nodeUrlHost: string,
		worldId: string,
		backupId: string,
		token: string,
	): string {
		const url = new URL(
			`/v1/worlds/${worldId}/backups/${backupId}/download`,
			getNodeBaseUrl(nodeUrlHost),
		)
		url.searchParams.set('token', token)
		return url.toString()
	}
}
