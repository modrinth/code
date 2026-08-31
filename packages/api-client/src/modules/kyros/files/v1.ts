import { AbstractModule } from '../../../core/abstract-module'
import type { Kyros } from '../types'

export class KyrosFilesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v1'
	}

	/**
	 * Download the complete filesystem for a world as a ZIP archive.
	 */
	public async downloadWorldZip(worldId: string): Promise<Blob> {
		return this.client.request<Blob>(`/worlds/${worldId}/files/contents-zip`, {
			api: '',
			version: 'v1',
			method: 'GET',
			useNodeAuth: true,
		})
	}

	/**
	 * Create a ZIP archive beside a directory in world storage.
	 */
	public async zipFolder(
		worldId: string,
		data: Kyros.Files.v1.ZipFolderRequest,
	): Promise<Kyros.Files.v1.FileMutationResponse> {
		return this.client.request<Kyros.Files.v1.FileMutationResponse>(
			`/worlds/${worldId}/files/zip`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				body: data,
				useNodeAuth: true,
			},
		)
	}
}
