import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

export class LabrinthModerationInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_moderation_internal'
	}

	public async acquireLock(
		projectId: string,
	): Promise<Labrinth.Moderation.Internal.LockAcquireResponse> {
		return this.client.request<Labrinth.Moderation.Internal.LockAcquireResponse>(
			`/moderation/lock/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
			},
		)
	}

	public async overrideLock(
		projectId: string,
	): Promise<Labrinth.Moderation.Internal.LockAcquireResponse> {
		return this.client.request<Labrinth.Moderation.Internal.LockAcquireResponse>(
			`/moderation/lock/${projectId}/override`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
			},
		)
	}

	public async releaseLock(
		projectId: string,
	): Promise<Labrinth.Moderation.Internal.ReleaseLockResponse> {
		return this.client.request<Labrinth.Moderation.Internal.ReleaseLockResponse>(
			`/moderation/lock/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'DELETE',
			},
		)
	}

	public async checkLock(
		projectId: string,
	): Promise<Labrinth.Moderation.Internal.LockStatusResponse> {
		return this.client.request<Labrinth.Moderation.Internal.LockStatusResponse>(
			`/moderation/lock/${projectId}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async setProjectJudgements(
		judgements: Labrinth.Moderation.Internal.ProjectJudgements,
	): Promise<void> {
		return this.client.request<void>('/moderation/project', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: judgements,
		})
	}
}
