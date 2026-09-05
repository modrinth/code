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
	 * Get a user's account preferences. The authenticated user may access their
	 * own preferences, while moderators may access another user's preferences.
	 *
	 * GET /v3/user/{id}/preferences
	 */
	public async getPreferences(idOrUsername: string): Promise<Labrinth.Users.v3.UserPreferences> {
		return this.client.request<Labrinth.Users.v3.UserPreferences>(
			`/user/${encodeURIComponent(idOrUsername)}/preferences`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Update a user's account preferences and return the fully resolved result.
	 *
	 * PATCH /v3/user/{id}/preferences
	 */
	public async patchPreferences(
		idOrUsername: string,
		preferences: Labrinth.Users.v3.PartialUserPreferences,
	): Promise<Labrinth.Users.v3.UserPreferences> {
		return this.client.request<Labrinth.Users.v3.UserPreferences>(
			`/user/${encodeURIComponent(idOrUsername)}/preferences`,
			{
				api: 'labrinth',
				version: 3,
				method: 'PATCH',
				body: preferences,
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
	 * Get a user's projects.
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of the user's projects
	 *
	 * GET /v3/user/{id}/projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.users_v3.getProjects('my_user')
	 * ```
	 */
	public async getProjects(idOrUsername: string): Promise<Labrinth.Projects.v3.Project[]> {
		return this.client.request<Labrinth.Projects.v3.Project[]>(
			`/user/${encodeURIComponent(idOrUsername)}/projects`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get projects a user follows.
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of followed projects
	 *
	 * GET /v3/user/{id}/follows
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.users_v3.getFollowedProjects('my_user')
	 * ```
	 */
	public async getFollowedProjects(idOrUsername: string): Promise<Labrinth.Projects.v3.Project[]> {
		return this.client.request<Labrinth.Projects.v3.Project[]>(
			`/user/${encodeURIComponent(idOrUsername)}/follows`,
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
