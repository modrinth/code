import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthCopyrightArchivesInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_copyright_archives_internal'
	}

	public async list(
		params: { limit?: number; offset?: number } = {},
	): Promise<Labrinth.CopyrightArchives.Internal.Archive[]> {
		return this.client.request('/moderation/copyright-archives', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			params,
		})
	}

	public async create(archives: Labrinth.CopyrightArchives.Internal.Create[]): Promise<string[]> {
		return this.client.request('/moderation/copyright-archives', {
			api: 'labrinth',
			version: 'internal',
			method: 'PUT',
			body: archives,
			retry: false,
		})
	}

	public async replace(archives: Labrinth.CopyrightArchives.Internal.Replace[]): Promise<void> {
		return this.client.request('/moderation/copyright-archives', {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: archives,
		})
	}

	public async delete(ids: string[]): Promise<void> {
		return this.client.request('/moderation/copyright-archives', {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
			body: ids,
		})
	}
}
