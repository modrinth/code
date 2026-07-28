import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthBlockedUsersV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_blocked_users_v3'
	}

	/**
	 * List the users blocked by the authenticated user.
	 */
	public async list(): Promise<Labrinth.BlockedUsers.v3.BlockedUserId[]> {
		return this.client.request<Labrinth.BlockedUsers.v3.BlockedUserId[]>('/blocks', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Block a user.
	 *
	 * @param idOrUsername - The target user's ID or username
	 */
	public async block(idOrUsername: string): Promise<void> {
		return this.client.request(`/block/${encodeURIComponent(idOrUsername)}`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
		})
	}

	/**
	 * Unblock a user.
	 *
	 * @param idOrUsername - The target user's ID or username
	 */
	public async unblock(idOrUsername: string): Promise<void> {
		return this.client.request(`/block/${encodeURIComponent(idOrUsername)}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
