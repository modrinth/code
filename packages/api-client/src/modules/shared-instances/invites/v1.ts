import { AbstractModule } from '../../../core/abstract-module'
import type { SharedInstances } from '../types'

export class SharedInstancesInvitesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'sharedinstances_invites_v1'
	}

	public async get(inviteId: string): Promise<SharedInstances.Invites.v1.Invite> {
		return this.client.request<SharedInstances.Invites.v1.Invite>(
			`/invites/${encodeURIComponent(inviteId)}`,
			{
				api: 'sharedinstances',
				version: 1,
				method: 'GET',
				skipAuth: true,
				retry: false,
			},
		)
	}

	public async list(instanceId: string): Promise<SharedInstances.Invites.v1.InviteLink[]> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/invites`, {
			api: 'sharedinstances', version: 1, method: 'GET',
		})
	}

	public async create(instanceId: string, options: { max_age?: number; max_uses: number }): Promise<{ id: string }> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/invites`, {
			api: 'sharedinstances', version: 1, method: 'POST', body: options, retry: false,
		})
	}

	public async delete(instanceId: string, inviteId: string): Promise<void> {
		return this.client.request(`/instances/${encodeURIComponent(instanceId)}/invites/${encodeURIComponent(inviteId)}`, {
			api: 'sharedinstances', version: 1, method: 'DELETE',
		})
	}
}
