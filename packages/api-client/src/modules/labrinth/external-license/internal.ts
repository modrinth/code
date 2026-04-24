import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthExternalLicenseInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_external_license_internal'
	}

	public async search(
		params: Labrinth.ExternalLicense.Internal.SearchRequest,
	): Promise<Labrinth.ExternalLicense.Internal.ExternalProject[]> {
		return this.client.request<Labrinth.ExternalLicense.Internal.ExternalProject[]>(
			'/moderation/external-license/search',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: params,
			},
		)
	}

	public async getBySha1(sha1: string): Promise<Labrinth.ExternalLicense.Internal.ExternalProject> {
		return this.client.request<Labrinth.ExternalLicense.Internal.ExternalProject>(
			`/moderation/external-license/by-sha1/${sha1}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async updateLicense(
		id: number,
		data: Labrinth.ExternalLicense.Internal.UpdateLicenseRequest,
	): Promise<Labrinth.ExternalLicense.Internal.ExternalProject> {
		return this.client.request<Labrinth.ExternalLicense.Internal.ExternalProject>(
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
