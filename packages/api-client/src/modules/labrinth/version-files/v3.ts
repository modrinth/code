import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthVersionFilesV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_version_files_v3'
	}

	public async getProjectsByHashes(
		algorithm: 'sha1' | 'sha512',
		hashes: string[],
	): Promise<Record<string, Labrinth.Projects.v3.Project>> {
		return this.client.request<Record<string, Labrinth.Projects.v3.Project>>(
			`/version_file/project`,
			{
				api: 'labrinth',
				version: 3,
				method: 'POST',
				body: {
					algorithm,
					hashes,
				},
			},
		)
	}
}
