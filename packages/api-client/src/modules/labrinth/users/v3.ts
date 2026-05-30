import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthUsersV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_users_v3'
	}

	/**
	 * Get the authenticated user.
	 * GET /v3/user
	 */
	public async getAuthenticated(): Promise<Labrinth.Users.v3.User> {
		return this.client.request<Labrinth.Users.v3.User>('/user', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}
}
