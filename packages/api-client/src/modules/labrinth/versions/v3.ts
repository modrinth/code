import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthVersionsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_versions_v3'
	}

	/**
	 * Get versions for a project (v3)
	 *
	 * @param id - Project ID or slug (e.g., 'sodium' or 'AANobbMI')
	 * @param options - Optional query parameters to filter versions
	 * @returns Promise resolving to an array of v3 versions
	 *
	 * @example
	 * ```typescript
	 * const versions = await client.labrinth.versions_v3.getProjectVersions('sodium')
	 * const filteredVersions = await client.labrinth.versions_v3.getProjectVersions('sodium', {
	 *   game_versions: ['1.20.1'],
	 *   loaders: ['fabric']
	 * })
	 * console.log(versions[0].version_number)
	 * ```
	 */
	public async getProjectVersions(
		id: string,
		options?: Labrinth.Versions.v3.GetProjectVersionsParams,
	): Promise<Labrinth.Versions.v3.Version[]> {
		const params: Record<string, string> = {}
		if (options?.game_versions?.length) {
			params.game_versions = JSON.stringify(options.game_versions)
		}
		if (options?.loaders?.length) {
			params.loaders = JSON.stringify(options.loaders)
		}

		return this.client.request<Labrinth.Versions.v3.Version[]>(`/project/${id}/version`, {
			api: 'labrinth',
			version: 2, // TODO: move this to a versions v2 module to keep api-client clean and organized
			method: 'GET',
			params: Object.keys(params).length > 0 ? params : undefined,
		})
	}

	/**
	 * Get a specific version by ID (v3)
	 *
	 * @param id - Version ID
	 * @returns Promise resolving to the v3 version data
	 *
	 * @example
	 * ```typescript
	 * const version = await client.labrinth.versions_v3.getVersion('DXtmvS8i')
	 * console.log(version.version_number)
	 * ```
	 */
	public async getVersion(id: string): Promise<Labrinth.Versions.v3.Version> {
		return this.client.request<Labrinth.Versions.v3.Version>(`/version/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get multiple versions by IDs (v3)
	 *
	 * @param ids - Array of version IDs
	 * @returns Promise resolving to an array of v3 versions
	 *
	 * @example
	 * ```typescript
	 * const versions = await client.labrinth.versions_v3.getVersions(['DXtmvS8i', 'abc123'])
	 * console.log(versions[0].version_number)
	 * ```
	 */
	public async getVersions(ids: string[]): Promise<Labrinth.Versions.v3.Version[]> {
		return this.client.request<Labrinth.Versions.v3.Version[]>(`/versions`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: { ids: JSON.stringify(ids) },
		})
	}

	/**
	 * Get a version from a project by version ID or number (v3)
	 *
	 * @param projectId - Project ID or slug
	 * @param versionId - Version ID or version number
	 * @returns Promise resolving to the v3 version data
	 *
	 * @example
	 * ```typescript
	 * const version = await client.labrinth.versions_v3.getVersionFromIdOrNumber('sodium', 'DXtmvS8i')
	 * const versionByNumber = await client.labrinth.versions_v3.getVersionFromIdOrNumber('sodium', '0.4.12')
	 * ```
	 */
	public async getVersionFromIdOrNumber(
		projectId: string,
		versionId: string,
	): Promise<Labrinth.Versions.v3.Version> {
		return this.client.request<Labrinth.Versions.v3.Version>(
			`/project/${projectId}/version/${versionId}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Create a new version for a project (v3)
	 *
	 * Creates a new version on an existing project. At least one file must be
	 * attached unless the version is created as a draft.
	 *
	 * @param data - JSON metadata payload for the version (must include file_parts)
	 * @param files - Array of uploaded files, in the same order as `data.file_parts`
	 *
	 * @returns A promise resolving to the newly created version data
	 *
	 * @example
	 * ```ts
	 * const version = await client.labrinth.versions_v3.createVersion('sodium', {
	 *   name: 'v0.5.0',
	 *   version_number: '0.5.0',
	 *   version_type: 'release',
	 *   loaders: ['fabric'],
	 *   game_versions: ['1.20.1'],
	 *   project_id: 'sodium',
	 *   file_parts: ['primary']
	 * }, [fileObject])
	 * ```
	 */

	public async createVersion(
		draftVersion: Labrinth.Versions.v3.DraftVersion,
		versionFiles: Labrinth.Versions.v3.DraftVersionFile[],
		projectType: Labrinth.Projects.v2.ProjectType | null = null,
	): Promise<Labrinth.Versions.v3.Version> {
		const formData = new FormData()

		const files = versionFiles.map((vf) => vf.file)
		const fileTypes = versionFiles.map((vf) => vf.fileType || null)

		const fileParts = files.map((file, i) => {
			return `${file.name}-${i === 0 ? 'primary' : i}`
		})

		const fileTypeMap = fileParts.reduce<Record<string, Labrinth.Versions.v3.FileType | null>>(
			(acc, key, i) => {
				acc[key] = fileTypes[i]
				return acc
			},
			{},
		)

		const data: Labrinth.Versions.v3.CreateVersionRequest = {
			project_id: draftVersion.project_id,
			version_number: draftVersion.version_number,
			name: draftVersion.name || draftVersion.version_number,
			changelog: draftVersion.changelog,
			dependencies: draftVersion.dependencies || [],
			game_versions: draftVersion.game_versions,
			version_type: draftVersion.version_type,
			featured: !!draftVersion.featured,
			file_parts: fileParts,
			file_types: fileTypeMap,
			primary_file: fileParts[0],
			environment: draftVersion.environment,
			loaders: draftVersion.loaders,
		}

		if (projectType === 'modpack') {
			data.mrpack_loaders = draftVersion.loaders
			data.loaders = ['mrpack']
		}

		formData.append('data', JSON.stringify(data))

		files.forEach((file, i) => {
			formData.append(fileParts[i], new Blob([file]), file.name)
		})

		const newVersion = await this.client.request<Labrinth.Versions.v3.Version>(`/version`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: formData,
			timeout: 60 * 5 * 1000,
			headers: {
				'Content-Type': '',
			},
		})

		return newVersion
	}

	/**
	 * Modify an existing version by ID (v3)
	 *
	 * Partially updates a version’s metadata. Only JSON fields may be modified.
	 * To update files, use the separate "Add files to version" endpoint.
	 *
	 * @param versionId - The version ID to update
	 * @param data - PATCH metadata for this version (all fields optional)
	 *
	 * @returns A promise resolving to the updated version data
	 *
	 * @example
	 * ```ts
	 * const updated = await client.labrinth.versions_v3.modifyVersion('DXtmvS8i', {
	 *   name: 'v1.0.1',
	 *   changelog: 'Updated changelog',
	 *   featured: true,
	 *   status: 'listed'
	 * })
	 * ```
	 */

	public async modifyVersion(
		versionId: string,
		data: Labrinth.Versions.v3.ModifyVersionRequest,
	): Promise<Labrinth.Versions.v3.Version> {
		return this.client.request<Labrinth.Versions.v3.Version>(`/version/${versionId}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a version by ID (v3)
	 *
	 * @param versionId - Version ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.versions_v3.deleteVersion('DXtmvS8i')
	 * ```
	 */
	public async deleteVersion(versionId: string): Promise<void> {
		return this.client.request(`/version/${versionId}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}

	public async addFilesToVersion(
		versionId: string,
		versionFiles: Labrinth.Versions.v3.DraftVersionFile[],
	): Promise<Labrinth.Versions.v3.Version> {
		const formData = new FormData()

		const files = versionFiles.map((vf) => vf.file)
		const fileTypes = versionFiles.map((vf) => vf.fileType || null)

		const fileParts = files.map((file, i) => `${file.name}-${i}`)

		const fileTypeMap = fileParts.reduce<Record<string, Labrinth.Versions.v3.FileType | null>>(
			(acc, key, i) => {
				acc[key] = fileTypes[i]
				return acc
			},
			{},
		)

		formData.append('data', JSON.stringify({ file_types: fileTypeMap }))

		files.forEach((file, i) => {
			formData.append(fileParts[i], new Blob([file]), file.name)
		})

		return this.client.request<Labrinth.Versions.v3.Version>(`/version/${versionId}/file`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: formData,
			timeout: 60 * 5 * 1000,
			headers: {
				'Content-Type': '',
			},
		})
	}
}
