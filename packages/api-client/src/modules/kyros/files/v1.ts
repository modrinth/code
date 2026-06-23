import { AbstractModule } from '../../../core/abstract-module'
import type { Kyros } from '../types'

export class KyrosFilesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v1'
	}

	public async listDescendants(
		worldId: string,
		path: string,
		page: number = 1,
		itemsPerPage: number = 100,
	): Promise<Kyros.Files.v1.FileListingResponse> {
		return this.client.request<Kyros.Files.v1.FileListingResponse>(
			`/worlds/${worldId}/files/list`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				body: {
					path,
					page,
					items_per_page: itemsPerPage,
				},
				useNodeAuth: true,
			},
		)
	}

	public async downloadRawFileContents(worldId: string, path: string): Promise<Blob> {
		return this.client.request<Blob>(`/worlds/${worldId}/files/contents-raw`, {
			api: '',
			version: 'v1',
			method: 'GET',
			params: { path },
			useNodeAuth: true,
		})
	}

	public async downloadTokenizedContents(worldId: string, downloadId: string): Promise<Blob> {
		return this.client.request<Blob>(`/worlds/${worldId}/files/contents/${downloadId}`, {
			api: '',
			version: 'v1',
			method: 'GET',
			useNodeAuth: true,
		})
	}

	public async deleteFile(worldId: string, path: string): Promise<void> {
		return this.client.request<void>(`/worlds/${worldId}/files/delete`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: { path },
			useNodeAuth: true,
		})
	}

	public async moveFile(
		worldId: string,
		source: string,
		destination: string,
	): Promise<Kyros.Files.v1.FileMutationResponse> {
		return this.client.request<Kyros.Files.v1.FileMutationResponse>(
			`/worlds/${worldId}/files/move`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				body: { source, destination },
				useNodeAuth: true,
			},
		)
	}

	public async renameFile(
		worldId: string,
		path: string,
		name: string,
	): Promise<Kyros.Files.v1.FileMutationResponse> {
		return this.client.request<Kyros.Files.v1.FileMutationResponse>(
			`/worlds/${worldId}/files/rename`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				body: { path, name },
				useNodeAuth: true,
			},
		)
	}
}
