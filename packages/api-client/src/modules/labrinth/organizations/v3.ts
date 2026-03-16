import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthOrganizationsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_organizations_v3'
	}

	/**
	 * Get an organization by ID or slug
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @returns Promise resolving to the organization data
	 *
	 * @example
	 * ```typescript
	 * const org = await client.labrinth.organizations_v3.get('my-org')
	 * ```
	 */
	public async get(idOrSlug: string): Promise<Labrinth.Organizations.v3.Organization> {
		return this.client.request<Labrinth.Organizations.v3.Organization>(
			`/organization/${idOrSlug}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get an organization's projects
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @returns Promise resolving to the organization's projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.organizations_v3.getProjects('my-org')
	 * ```
	 */
	public async getProjects(idOrSlug: string): Promise<Labrinth.Projects.v3.Project[]> {
		return this.client.request<Labrinth.Projects.v3.Project[]>(
			`/organization/${idOrSlug}/projects`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}
}
