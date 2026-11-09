import { AbstractModule } from '../../../core/abstract-module'
import type { SharedInstances } from '../types'

export class SharedInstancesModerationV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'sharedinstances_moderation_v1'
	}

	public async blacklistUsers(
		request: SharedInstances.Moderation.v1.BlacklistUserRequest,
	): Promise<void> {
		return this.client.request('/moderation/blacklist', {
			api: 'sharedinstances',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	public async unblacklistUsers(
		request: SharedInstances.Moderation.v1.BlacklistUserRequest,
	): Promise<void> {
		return this.client.request('/moderation/blacklist', {
			api: 'sharedinstances',
			version: 1,
			method: 'DELETE',
			body: request,
		})
	}

	public async quarantineInstance(
		instanceId: string,
		request: SharedInstances.Moderation.v1.QuarantineInstanceRequest,
	): Promise<void> {
		return this.client.request(
			`/moderation/instances/${encodeURIComponent(instanceId)}/quarantine`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'PUT',
				body: request,
			},
		)
	}

	public async unquarantineInstance(instanceId: string): Promise<void> {
		return this.client.request(
			`/moderation/instances/${encodeURIComponent(instanceId)}/quarantine`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'DELETE',
			},
		)
	}

	/**
	 * @param fileType - Only needed when several files in the version share the name.
	 */
	public async deleteFile(
		instanceId: string,
		version: number,
		fileName: string,
		fileType?: string,
	): Promise<void> {
		return this.client.request(
			`/moderation/instances/${encodeURIComponent(instanceId)}/versions/${version}/files/${encodeURIComponent(fileName)}`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'DELETE',
				params: fileType ? { type: fileType } : undefined,
			},
		)
	}
}
