import { AbstractModule } from '../../../core/abstract-module'
import type { ProjectSearchParams, ProjectV2, SearchResult } from './types/v2'

export class LabrinthProjectsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_projects_v2'
	}

	/**
	 * Get a project by ID or slug
	 *
	 * @param id - Project ID or slug (e.g., 'sodium' or 'AANobbMI')
	 * @returns Promise resolving to the project data
	 *
	 * @example
	 * ```typescript
	 * const project = await client.labrinth.projects_v2.get('sodium')
	 * console.log(project.title) // "Sodium"
	 * ```
	 */
	public async get(id: string): Promise<ProjectV2> {
		return this.client.request<ProjectV2>(`/project/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Get multiple projects by IDs
	 *
	 * @param ids - Array of project IDs or slugs
	 * @returns Promise resolving to array of projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.projects_v2.getMultiple(['sodium', 'lithium', 'phosphor'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<ProjectV2[]> {
		return this.client.request<ProjectV2[]>(`/projects`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
			params: { ids: JSON.stringify(ids) },
		})
	}

	/**
	 * Search projects
	 *
	 * @param params - Search parameters (query, facets, filters, etc.)
	 * @returns Promise resolving to search results
	 *
	 * @example
	 * ```typescript
	 * const results = await client.labrinth.projects_v2.search({
	 *   query: 'optimization',
	 *   facets: [['categories:optimization'], ['project_type:mod']],
	 *   limit: 20
	 * })
	 * ```
	 */
	public async search(params: ProjectSearchParams): Promise<SearchResult> {
		return this.client.request<SearchResult>(`/search`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
			params: params as Record<string, unknown>,
		})
	}

	/**
	 * Edit a project
	 *
	 * @param id - Project ID or slug
	 * @param data - Project update data
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v2.edit('sodium', {
	 *   description: 'Updated description'
	 * })
	 * ```
	 */
	public async edit(id: string, data: Partial<ProjectV2>): Promise<void> {
		return this.client.request(`/project/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a project
	 *
	 * @param id - Project ID or slug
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v2.delete('my-project')
	 * ```
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/project/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}
}
