import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthModerationInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_moderation_internal'
	}

	public async getProjects(
		params: Labrinth.Moderation.Internal.ProjectsRequest = {},
	): Promise<Labrinth.Moderation.Internal.ProjectsResponse> {
		return this.client.request<Labrinth.Moderation.Internal.ProjectsResponse>(
			'/moderation/projects',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
				params,
			},
		)
	}

	public async getProjectIds(
		params: Omit<Labrinth.Moderation.Internal.ProjectsRequest, 'count' | 'offset'> = {},
	): Promise<Labrinth.Moderation.Internal.ProjectIdsResponse> {
		return this.client.request<Labrinth.Moderation.Internal.ProjectIdsResponse>(
			'/moderation/projects/ids',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
				params,
			},
		)
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
