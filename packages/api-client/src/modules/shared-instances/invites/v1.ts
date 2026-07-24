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
}
