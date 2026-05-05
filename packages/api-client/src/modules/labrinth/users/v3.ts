import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthUsersV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_users_v3'
	}

	/**
	 * Get a user's featured projects
	 *
	 * @param idOrUsername - The user's ID or username
	 * @returns Promise resolving to an array of featured projects
	 *
	 * @example
	 * ```typescript
	 * const featured = await client.labrinth.users_v3.getFeaturedProjects('my_user')
	 * ```
	 */
	public async getFeaturedProjects(idOrUsername: string): Promise<Labrinth.Projects.v2.Project[]> {
		return this.client.request<Labrinth.Projects.v2.Project[]>(`/user/${idOrUsername}/featured`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Toggle a project as featured for the current user
	 *
	 * @param userId - The ID of the user
	 * @param projectId - The project ID to toggle
	 * @returns Promise resolving when the operation is complete
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.users_v3.toggleFeaturedProject('user_id', 'project_id')
	 * ```
	 */
	public async toggleFeaturedProject(userId: string, projectId: string): Promise<void> {
		return this.client.request(`/user/${userId}/featured/${projectId}`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
		})
	}
}
