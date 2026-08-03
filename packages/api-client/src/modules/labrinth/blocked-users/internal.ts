import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthBlockedUsersInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_blocked_users_internal'
	}

	/**
	 * Check whether one user has blocked another.
	 */
	public async getStatus(
		userId: string,
		targetId: string,
	): Promise<Labrinth.BlockedUsers.Internal.BlockStatus> {
		return this.client.request<Labrinth.BlockedUsers.Internal.BlockStatus>(
			`/block/${encodeURIComponent(userId)}/${encodeURIComponent(targetId)}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}
}
