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
	 * Get multiple users by their IDs
	 *
	 * @param ids - Array of user IDs
	 * @returns Promise resolving to an array of users
	 *
	 * @example
	 * ```typescript
	 * const users = await client.labrinth.users_v2.getMultiple(['id1', 'id2'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Users.v2.User[]> {
		return this.client.request<Labrinth.Users.v2.User[]>(
			`/users?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 2,
				method: 'GET',
			},
		)
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

	/**
	 * Get a user's organizations
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of the user's organizations
	 *
	 * @example
	 * ```typescript
	 * const orgs = await client.labrinth.users_v2.getOrganizations('my_user')
	 * ```
	 */
	public async getOrganizations(
		idOrUsername: string,
	): Promise<Labrinth.Organizations.v3.Organization[]> {
		return this.client.request<Labrinth.Organizations.v3.Organization[]>(
			`/user/${idOrUsername}/organizations`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get a user's collections
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of the user's collections
	 *
	 * @example
	 * ```typescript
	 * const collections = await client.labrinth.users_v2.getCollections('my_user')
	 * ```
	 */
	public async getCollections(idOrUsername: string): Promise<Labrinth.Collections.Collection[]> {
		return this.client.request<Labrinth.Collections.Collection[]>(
			`/user/${idOrUsername}/collections`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Update a user
	 *
	 * @param idOrUsername - The user's ID or username
	 * @param data - Fields to update
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.users_v2.patch('my_user', { role: 'admin' })
	 * ```
	 */
	public async patch(
		idOrUsername: string,
		data: Partial<Pick<Labrinth.Users.v2.User, 'badges' | 'role'>>,
	): Promise<void> {
		return this.client.request(`/user/${idOrUsername}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}
}
