import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthUsersV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_users_v2'
	}

	/**
	 * Get a user by ID or username
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to the user data
	 *
	 * @example
	 * ```typescript
	 * const user = await client.labrinth.users_v2.get('my_user')
	 * ```
	 */
	public async get(idOrUsername: string): Promise<Labrinth.Users.v2.User> {
		return this.client.request<Labrinth.Users.v2.User>(`/user/${idOrUsername}`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Get a user's projects
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of the user's projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.users_v2.getProjects('my_user')
	 * ```
	 */
	public async getProjects(idOrUsername: string): Promise<Labrinth.Projects.v2.Project[]> {
		return this.client.request<Labrinth.Projects.v2.Project[]>(`/user/${idOrUsername}/projects`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}
}
