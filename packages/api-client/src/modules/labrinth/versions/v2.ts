import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthVersionsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_versions_v2'
	}

	/**
	 * Get versions for a project (v2)
	 *
	 * @param id - Project ID or slug (e.g., 'sodium' or 'AANobbMI')
	 * @param options - Optional query parameters to filter versions
	 * @returns Promise resolving to an array of v2 versions
	 *
	 * @example
	 * ```typescript
	 * const versions = await client.labrinth.versions_v2.getProjectVersions('sodium')
	 * const filteredVersions = await client.labrinth.versions_v2.getProjectVersions('sodium', {
	 *   game_versions: ['1.20.1'],
	 *   loaders: ['fabric'],
	 *   include_changelog: false
	 * })
	 * console.log(versions[0].version_number)
	 * ```
	 */
	public async getProjectVersions(
		id: string,
		options?: Labrinth.Versions.v2.GetProjectVersionsParams,
	): Promise<Labrinth.Versions.v2.Version[]> {
		const params: Record<string, string> = {}
		if (options?.game_versions?.length) {
			params.game_versions = JSON.stringify(options.game_versions)
		}
		if (options?.loaders?.length) {
			params.loaders = JSON.stringify(options.loaders)
		}
		if (options?.include_changelog === false) {
			params.include_changelog = 'false'
		}

		return this.client.request<Labrinth.Versions.v2.Version[]>(`/project/${id}/version`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
			params: Object.keys(params).length > 0 ? params : undefined,
		})
	}

	/**
	 * Get a specific version by ID (v2)
	 *
	 * @param id - Version ID
	 * @returns Promise resolving to the v2 version data
	 *
	 * @example
	 * ```typescript
	 * const version = await client.labrinth.versions_v2.getVersion('DXtmvS8i')
	 * console.log(version.version_number)
	 * ```
	 */
	public async getVersion(id: string): Promise<Labrinth.Versions.v2.Version> {
		return this.client.request<Labrinth.Versions.v2.Version>(`/version/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Get multiple versions by IDs (v2)
	 *
	 * @param ids - Array of version IDs
	 * @returns Promise resolving to an array of v2 versions
	 *
	 * @example
	 * ```typescript
	 * const versions = await client.labrinth.versions_v2.getVersions(['DXtmvS8i', 'abc123'])
	 * console.log(versions[0].version_number)
	 * ```
	 */
	public async getVersions(ids: string[]): Promise<Labrinth.Versions.v2.Version[]> {
		return this.client.request<Labrinth.Versions.v2.Version[]>(`/versions`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
			params: { ids: JSON.stringify(ids) },
		})
	}

	/**
	 * Get a version from a project by version ID or number (v2)
	 *
	 * @param projectId - Project ID or slug
	 * @param versionId - Version ID or version number
	 * @returns Promise resolving to the v2 version data
	 *
	 * @example
	 * ```typescript
	 * const version = await client.labrinth.versions_v2.getVersionFromIdOrNumber('sodium', 'DXtmvS8i')
	 * const versionByNumber = await client.labrinth.versions_v2.getVersionFromIdOrNumber('sodium', '0.4.12')
	 * ```
	 */
	public async getVersionFromIdOrNumber(
		projectId: string,
		versionId: string,
	): Promise<Labrinth.Versions.v2.Version> {
		return this.client.request<Labrinth.Versions.v2.Version>(
			`/project/${projectId}/version/${versionId}`,
			{
				api: 'labrinth',
				version: 2,
				method: 'GET',
			},
		)
	}

	/**
	 * Delete a version by ID (v2)
	 *
	 * @param versionId - Version ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.versions_v2.deleteVersion('DXtmvS8i')
	 * ```
	 */
	public async deleteVersion(versionId: string): Promise<void> {
		return this.client.request(`/version/${versionId}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}
}
