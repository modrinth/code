import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthUsersV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_users_v3'
	}

	/**
	 * Get all projects the authenticated user can access directly or through
	 * their organizations.
	 *
	 * GET /v3/all-projects
	 */
	public async getAllProjects(): Promise<Labrinth.Users.v3.AllProjectsResponse> {
		return this.client.request<Labrinth.Users.v3.AllProjectsResponse>('/all-projects', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}
}
