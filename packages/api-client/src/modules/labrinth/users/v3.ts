import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

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

	/**
	 * Get a user by ID or username
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to the user data
	 *
	 * GET /v3/user/{id}
	 */
	public async get(idOrUsername: string): Promise<Labrinth.Users.v3.User> {
		return this.client.request<Labrinth.Users.v3.User>(
			`/user/${encodeURIComponent(idOrUsername)}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Search users by username prefix.
	 *
	 * @param query - Username search query
	 * @returns Promise resolving to compact user search results
	 *
	 * GET /v3/users/search?query=:query
	 */
	public async search(query: string): Promise<Labrinth.Users.v3.SearchUser[]> {
		return this.client.request<Labrinth.Users.v3.SearchUser[]>(
			`/users/search?query=${encodeURIComponent(query)}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get all projects the authenticated user can access directly or through
	 * their organizations.
	 *
	 * @param idOrUsername - User ID or username. Must be the authenticated user.
	 *
	 * GET /v3/user/{id}/all-projects
	 */
	public async getAllProjects(
		idOrUsername: string,
	): Promise<Labrinth.Users.v3.AllProjectsResponse> {
		return this.client.request<Labrinth.Users.v3.AllProjectsResponse>(
			`/user/${encodeURIComponent(idOrUsername)}/all-projects`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}
}
