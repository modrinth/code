import type { AutoBackupSettings, Backup } from '@modrinth/utils'

import { useServersFetch } from '../servers-fetch.ts'
import { ServerModule } from './base.ts'

export class BackupsModule extends ServerModule {
	data: Backup[] = []

	async fetch(): Promise<void> {
		this.data = await useServersFetch<Backup[]>(`servers/${this.serverId}/backups`, {}, 'backups')
	}

	async create(backupName: string): Promise<string> {
		const response = await useServersFetch<{ id: string }>(`servers/${this.serverId}/backups`, {
			method: 'POST',
			body: { name: backupName },
		})
		await this.fetch() // Refresh this module
		return response.id
	}

	async rename(backupId: string, newName: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/rename`, {
			method: 'POST',
			body: { name: newName },
		})
		await this.fetch() // Refresh this module
	}

	async delete(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}`, {
			method: 'DELETE',
		})
		await this.fetch() // Refresh this module
	}

	async restore(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/restore`, {
			method: 'POST',
		})
		await this.fetch() // Refresh this module
	}

	async prepare(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/prepare-download`, {
			method: 'POST',
		})
	}

	async lock(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/lock`, {
			method: 'POST',
		})
		await this.fetch() // Refresh this module
	}

	async unlock(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/unlock`, {
			method: 'POST',
		})
		await this.fetch() // Refresh this module
	}

	async retry(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/retry`, {
			method: 'POST',
		})
	}

	async updateAutoBackup(autoBackup: 'enable' | 'disable', interval: number): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/autobackup`, {
			method: 'POST',
			body: { set: autoBackup, interval },
		})
	}

	async getAutoBackup(): Promise<AutoBackupSettings> {
		return await useServersFetch(`servers/${this.serverId}/autobackup`)
	}
}
