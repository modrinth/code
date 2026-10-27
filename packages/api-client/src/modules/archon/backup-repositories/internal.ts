import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonBackupRepositoriesInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'archon_backup_repositories_internal'
	}

	/**
	 * List the latest reparation runs of a backup repository with their operations, newest first.
	 * GET /_internal/backups/repositories/:id/reparation
	 */
	public async getReparations(
		repositoryId: number,
	): Promise<Archon.BackupRepositories.Internal.ReparationRun[]> {
		return this.client.request<Archon.BackupRepositories.Internal.ReparationRun[]>(
			`/backups/repositories/${repositoryId}/reparation`,
			{
				api: 'archon',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Start a reparation run of a backup repository by checking it.
	 * POST /_internal/backups/repositories/:id/reparation
	 */
	public async startReparation(
		repositoryId: number,
		options?: Archon.BackupRepositories.Internal.StartReparationQuery,
	): Promise<Archon.BackupRepositories.Internal.ReparationRun> {
		return this.client.request<Archon.BackupRepositories.Internal.ReparationRun>(
			`/backups/repositories/${repositoryId}/reparation`,
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
				params: options?.read ? { read: true } : undefined,
			},
		)
	}

	/**
	 * Start an operation in the backup repository's latest reparation run.
	 * POST /_internal/backups/repositories/:id/reparation/:operation
	 */
	public async startOperation(
		repositoryId: number,
		operationType: Archon.BackupRepositories.Internal.RunnableOperationType,
	): Promise<Archon.BackupRepositories.Internal.ReparationOperation> {
		return this.client.request<Archon.BackupRepositories.Internal.ReparationOperation>(
			`/backups/repositories/${repositoryId}/reparation/${operationType}`,
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
			},
		)
	}
}
