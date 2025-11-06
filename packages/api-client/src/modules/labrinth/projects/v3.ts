import { AbstractModule } from '../../../core/abstract-module'
import type { ProjectV3 } from './types/v3'

export class LabrinthProjectsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_projects_v3'
	}

	/**
	 * Get a project by ID or slug (v3)
	 *
	 * @param id - Project ID or slug (e.g., 'sodium' or 'AANobbMI')
	 * @returns Promise resolving to the v3 project data
	 *
	 * @example
	 * ```typescript
	 * const project = await client.labrinth.projects_v3.get('sodium')
	 * console.log(project.project_types) // v3 field
	 * ```
	 */
	public async get(id: string): Promise<ProjectV3> {
		return this.client.request<ProjectV3>(`/project/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get multiple projects by IDs (v3)
	 *
	 * @param ids - Array of project IDs or slugs
	 * @returns Promise resolving to array of v3 projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.projects_v3.getMultiple(['sodium', 'lithium'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<ProjectV3[]> {
		return this.client.request<ProjectV3[]>(`/projects`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: { ids: JSON.stringify(ids) },
		})
	}

	/**
	 * Edit a project (v3)
	 *
	 * @param id - Project ID or slug
	 * @param data - Project update data (v3 fields)
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v3.edit('sodium', {
	 *   environment: 'client_and_server'
	 * })
	 * ```
	 */
	public async edit(id: string, data: Partial<ProjectV3>): Promise<void> {
		return this.client.request(`/project/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}
}
