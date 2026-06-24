import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthExternalProjectsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_external_projects_internal'
	}

	public async search(
		data: Labrinth.ExternalProjects.Internal.SearchRequest,
	): Promise<Labrinth.ExternalProjects.Internal.ExternalProject[]> {
		return this.client.request<Labrinth.ExternalProjects.Internal.ExternalProject[]>(
			'/moderation/external-license/search',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: data,
			},
		)
	}

	public async getBySha1(
		sha1: string,
	): Promise<Labrinth.ExternalProjects.Internal.ExternalProject> {
		return this.client.request<Labrinth.ExternalProjects.Internal.ExternalProject>(
			`/moderation/external-license/by-sha1/${sha1}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async update(
		id: number,
		data: Labrinth.ExternalProjects.Internal.UpdateLicenseRequest,
	): Promise<Labrinth.ExternalProjects.Internal.ExternalProject> {
		return this.client.request<Labrinth.ExternalProjects.Internal.ExternalProject>(
			`/moderation/external-license/${id}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'PATCH',
				body: data,
			},
		)
	}
}
