import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'

export class KyrosFilesV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v0'
	}

	/**
	 * Download a file from a server's filesystem
	 *
	 * @param nodeInstance - Node instance URL (e.g., "node-xyz.modrinth.com/modrinth/v0/fs")
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - File path (e.g., "/server-icon-original.png")
	 * @returns Promise resolving to file Blob
	 */
	public async downloadFile(nodeInstance: string, nodeToken: string, path: string): Promise<Blob> {
		return this.client.request<Blob>(`/fs/download`, {
			api: `https://${nodeInstance.replace('v0/fs', '')}`,
			method: 'GET',
			version: 'v0',
			params: { path },
			headers: { Authorization: `Bearer ${nodeToken}` },
		})
	}

	/**
	 * Upload a file to a server's filesystem with progress tracking
	 *
	 * @param nodeInstance - Node instance URL (e.g., "node-xyz.modrinth.com/modrinth/v0/fs")
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Destination path (e.g., "/server-icon.png")
	 * @param file - File to upload
	 * @param options - Optional progress callback and feature overrides
	 * @returns UploadHandle with promise, onProgress, and cancel
	 */
	public uploadFile(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		file: File,
		options?: {
			onProgress?: (progress: UploadProgress) => void
			retry?: boolean | number
		},
	): UploadHandle<void> {
		const baseUrl = `https://${nodeInstance.replace('v0/fs', '')}`

		return this.client.upload<void>('/fs/create', {
			api: baseUrl,
			version: 'v0',
			file,
			params: { path, type: 'file' },
			headers: {
				Authorization: `Bearer ${nodeToken}`,
			},
			onProgress: options?.onProgress,
			retry: options?.retry,
			skipAuth: true, // Use nodeToken, not main auth
		})
	}
}
