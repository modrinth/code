import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import type { Kyros } from '../types'

export class KyrosFilesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v1'
	}

	private isConflict(error: unknown): boolean {
		const err = error as { statusCode?: number; response?: { status?: number } }
		return (err.statusCode ?? err.response?.status) === 409
	}

	public async createDownloadSession(
		worldId: string,
		path: string,
		zipped: boolean,
	): Promise<void> {
		return this.client.request<void>(`/worlds/${worldId}/files/contents`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: { path, zipped },
			useNodeAuth: true,
		})
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

	public async editFile(worldId: string, path: string, content: string | Blob): Promise<void> {
		const body = typeof content === 'string' ? new Blob([content]) : content

		return this.client.request<void>(`/worlds/${worldId}/files/edit`, {
			api: '',
			version: 'v1',
			method: 'POST',
			params: { path },
			body,
			headers: { 'Content-Type': 'application/octet-stream' },
			useNodeAuth: true,
		})
	}

	public uploadFile(
		worldId: string,
		path: string,
		file: File | Blob,
		options?: {
			onProgress?: (progress: UploadProgress) => void
			retry?: boolean | number
		},
	): UploadHandle<void> {
		return this.client.upload<void>(`/worlds/${worldId}/files/edit`, {
			api: '',
			version: 'v1',
			file,
			params: { path },
			onProgress: options?.onProgress,
			retry: options?.retry,
			useNodeAuth: true,
		})
	}

	public async touchFile(worldId: string, path: string): Promise<void> {
		return this.client.request<void>(`/worlds/${worldId}/files/touch`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: { path },
			useNodeAuth: true,
		})
	}

	public async mkdirFile(worldId: string, path: string): Promise<void> {
		return this.client.request<void>(`/worlds/${worldId}/files/mkdir`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: { path },
			useNodeAuth: true,
		})
	}

	public async ensureFile(worldId: string, path: string): Promise<void> {
		try {
			await this.touchFile(worldId, path)
		} catch (error) {
			if (!this.isConflict(error)) {
				throw error
			}
		}
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

	public unzipFile(
		worldId: string,
		request: Kyros.Files.v1.UnzipFileRequest,
	): Promise<ReadableStream<Uint8Array>> {
		return this.client.stream(`/worlds/${worldId}/files/unzip`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: request,
			headers: { Accept: 'application/json-seq' },
			useNodeAuth: true,
		})
	}

	public uploadZip(
		worldId: string,
		path: string,
		file: File | Blob,
		options?: {
			onProgress?: (progress: UploadProgress) => void
			retry?: boolean | number
		},
	): UploadHandle<void> {
		return this.client.upload<void>(`/worlds/${worldId}/files/upload-zip`, {
			api: '',
			version: 'v1',
			file,
			params: { path },
			headers: { 'Content-Type': 'application/zip' },
			onProgress: options?.onProgress,
			retry: options?.retry,
			useNodeAuth: true,
		})
	}

	public async modifyOperation(opId: string, action: 'dismiss' | 'cancel'): Promise<void> {
		return this.client.request<void>(`/fs/ops/${action}`, {
			api: '',
			version: 'v1',
			method: 'POST',
			params: { id: opId },
			useNodeAuth: true,
		})
	}
}
