import type { AutoBackupSettings, Backup } from '@modrinth/utils'

import { useServersFetch } from '../servers-fetch.ts'
import { ServerModule } from './base.ts'

export class BackupsModule extends ServerModule {
	data: Backup[] = []

	async fetch(): Promise<void> {
		this.data = await useServersFetch<Backup[]>(`servers/${this.serverId}/backups`, {}, 'backups')
	}

	async create(backupName: string): Promise<string> {
		const tempId = `temp-${Date.now()}-${Math.random().toString(36).substring(7)}`
		const tempBackup: Backup = {
			id: tempId,
			name: backupName,
			created_at: new Date().toISOString(),
			locked: false,
			automated: false,
			interrupted: false,
			ongoing: true,
			task: { create: { progress: 0, state: 'ongoing' } },
		}
		this.data.push(tempBackup)

		try {
			const response = await useServersFetch<{ id: string }>(`servers/${this.serverId}/backups`, {
				method: 'POST',
				body: { name: backupName },
			})

			const backup = this.data.find((b) => b.id === tempId)
			if (backup) {
				backup.id = response.id
			}

			return response.id
		} catch (error) {
			this.data = this.data.filter((b) => b.id !== tempId)
			throw error
		}
	}

	async rename(backupId: string, newName: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/rename`, {
			method: 'POST',
			body: { name: newName },
		})
		await this.fetch()
	}

	async delete(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}`, {
			method: 'DELETE',
		})
		await this.fetch()
	}

	async restore(backupId: string): Promise<void> {
		const backup = this.data.find((b) => b.id === backupId)
		if (backup) {
			if (!backup.task) backup.task = {}
			backup.task.restore = { progress: 0, state: 'ongoing' }
		}

		try {
			await useServersFetch(`servers/${this.serverId}/backups/${backupId}/restore`, {
				method: 'POST',
			})
		} catch (error) {
			if (backup?.task?.restore) {
				delete backup.task.restore
			}
			throw error
		}
	}

	async lock(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/lock`, {
			method: 'POST',
		})
		await this.fetch()
	}

	async unlock(backupId: string): Promise<void> {
		await useServersFetch(`servers/${this.serverId}/backups/${backupId}/unlock`, {
			method: 'POST',
		})
		await this.fetch()
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
