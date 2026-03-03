import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

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
	public async get(id: string): Promise<Labrinth.Projects.v2.Project> {
		return this.client.request<Labrinth.Projects.v2.Project>(`/project/${id}`, {
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
	public async getMultiple(ids: string[]): Promise<Labrinth.Projects.v2.Project[]> {
		return this.client.request<Labrinth.Projects.v2.Project[]>(`/projects`, {
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
	public async search(
		params: Labrinth.Projects.v2.ProjectSearchParams,
	): Promise<Labrinth.Projects.v2.SearchResult> {
		return this.client.request<Labrinth.Projects.v2.SearchResult>(`/search`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
			params: {
				...params,
				facets: params.facets ? JSON.stringify(params.facets) : undefined,
			},
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
	public async edit(id: string, data: Partial<Labrinth.Projects.v2.Project>): Promise<void> {
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

	/**
	 * Get dependencies for a project
	 *
	 * @param id - Project ID or slug
	 * @returns Promise resolving to dependency info (projects and versions)
	 *
	 * @example
	 * ```typescript
	 * const deps = await client.labrinth.projects_v2.getDependencies('sodium')
	 * console.log(deps.projects) // dependent projects
	 * console.log(deps.versions) // dependent versions
	 * ```
	 */
	public async getDependencies(id: string): Promise<Labrinth.Projects.v2.DependencyInfo> {
		return this.client.request<Labrinth.Projects.v2.DependencyInfo>(`/project/${id}/dependencies`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Create a gallery image for a project
	 *
	 * @param id - Project ID or slug
	 * @param file - Image file to upload
	 * @param options - Gallery image options
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v2.createGalleryImage('sodium', imageFile, {
	 *   featured: true,
	 *   title: 'Screenshot 1',
	 *   description: 'Main menu with Sodium enabled'
	 * })
	 * ```
	 */
	public async createGalleryImage(
		id: string,
		file: Blob,
		options: {
			ext: string
			featured: boolean
			title?: string
			description?: string
			ordering?: number
		},
	): Promise<void> {
		const params: Record<string, string> = {
			ext: options.ext,
			featured: String(options.featured),
		}
		if (options.title) params.title = options.title
		if (options.description) params.description = options.description
		if (options.ordering !== undefined) params.ordering = String(options.ordering)

		return this.client.request(`/project/${id}/gallery`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			params,
			body: file,
		})
	}

	/**
	 * Edit a gallery image for a project
	 *
	 * @param id - Project ID or slug
	 * @param url - URL of the existing gallery image to edit
	 * @param options - Gallery image options to update
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v2.editGalleryImage('sodium', 'https://cdn.modrinth.com/...', {
	 *   featured: false,
	 *   title: 'Updated title'
	 * })
	 * ```
	 */
	public async editGalleryImage(
		id: string,
		url: string,
		options: {
			featured: boolean
			title?: string
			description?: string
			ordering?: number
		},
	): Promise<void> {
		const params: Record<string, string> = {
			url,
			featured: String(options.featured),
		}
		if (options.title) params.title = options.title
		if (options.description) params.description = options.description
		if (options.ordering !== undefined) params.ordering = String(options.ordering)

		return this.client.request(`/project/${id}/gallery`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			params,
		})
	}

	/**
	 * Delete a gallery image from a project
	 *
	 * @param id - Project ID or slug
	 * @param url - URL of the gallery image to delete
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.projects_v2.deleteGalleryImage('sodium', 'https://cdn.modrinth.com/...')
	 * ```
	 */
	public async deleteGalleryImage(id: string, url: string): Promise<void> {
		return this.client.request(`/project/${id}/gallery`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
			params: { url },
		})
	}
}
