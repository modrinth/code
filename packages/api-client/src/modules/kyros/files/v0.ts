import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import type { Kyros } from '../types'

export class KyrosFilesV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v0'
	}

	/**
	 * Get base URL from node instance
	 */
	private getBaseUrl(nodeInstance: string): string {
		return `https://${nodeInstance.replace('v0/fs', '')}`
	}

	/**
	 * List directory contents with pagination
	 *
	 * @param nodeInstance - Node instance URL (e.g., "node-xyz.modrinth.com/modrinth/v0/fs")
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Directory path (e.g., "/")
	 * @param page - Page number (1-indexed)
	 * @param pageSize - Items per page
	 * @returns Directory listing with items and pagination info
	 */
	public async listDirectory(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		page: number = 1,
		pageSize: number = 100,
	): Promise<Kyros.Files.v0.DirectoryResponse> {
		return this.client.request<Kyros.Files.v0.DirectoryResponse>('/fs/list', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'GET',
			params: { path, page, page_size: pageSize },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
		})
	}

	/**
	 * Create a file or directory
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Path for new item (e.g., "/new-folder")
	 * @param type - Type of item to create
	 */
	public async createFileOrFolder(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		type: 'file' | 'directory',
	): Promise<void> {
		return this.client.request<void>('/fs/create', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'POST',
			params: { path, type },
			headers: {
				Authorization: `Bearer ${nodeToken}`,
				'Content-Type': 'application/octet-stream',
			},
			skipAuth: true,
		})
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
		return this.client.request<Blob>('/fs/download', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'GET',
			params: { path },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
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
		file: File | Blob,
		options?: {
			onProgress?: (progress: UploadProgress) => void
			retry?: boolean | number
		},
	): UploadHandle<void> {
		return this.client.upload<void>('/fs/create', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			file,
			params: { path, type: 'file' },
			headers: {
				Authorization: `Bearer ${nodeToken}`,
			},
			onProgress: options?.onProgress,
			retry: options?.retry,
			skipAuth: true,
		})
	}

	/**
	 * Update file contents
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - File path to update
	 * @param content - New file content (string or Blob)
	 */
	public async updateFile(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		content: string | Blob,
	): Promise<void> {
		const blob = typeof content === 'string' ? new Blob([content]) : content

		return this.client.request<void>('/fs/update', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'PUT',
			params: { path },
			body: blob,
			headers: {
				Authorization: `Bearer ${nodeToken}`,
				'Content-Type': 'application/octet-stream',
			},
			skipAuth: true,
		})
	}

	/**
	 * Move a file or folder to a new location
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param sourcePath - Current path
	 * @param destPath - New path
	 */
	public async moveFileOrFolder(
		nodeInstance: string,
		nodeToken: string,
		sourcePath: string,
		destPath: string,
	): Promise<void> {
		return this.client.request<void>('/fs/move', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'POST',
			body: { source: sourcePath, destination: destPath },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
		})
	}

	/**
	 * Rename a file or folder (convenience wrapper around move)
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Current file/folder path
	 * @param newName - New name (not full path)
	 */
	public async renameFileOrFolder(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		newName: string,
	): Promise<void> {
		const newPath = path.split('/').slice(0, -1).join('/') + '/' + newName
		return this.moveFileOrFolder(nodeInstance, nodeToken, path, newPath)
	}

	/**
	 * Delete a file or folder
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Path to delete
	 * @param recursive - If true, delete directory contents recursively
	 */
	public async deleteFileOrFolder(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		recursive: boolean,
	): Promise<void> {
		return this.client.request<void>('/fs/delete', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v0',
			method: 'DELETE',
			params: { path, recursive },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
		})
	}

	/**
	 * Extract an archive file (zip, tar, etc.)
	 *
	 * Uses v1 API endpoint.
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Path to archive file
	 * @param override - If true, overwrite existing files
	 * @param dry - If true, perform dry run (returns conflicts without extracting)
	 * @returns Extract result with modpack name and conflicting files
	 */
	public async extractFile(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		override: boolean = true,
		dry: boolean = false,
	): Promise<Kyros.Files.v0.ExtractResult> {
		return this.client.request<Kyros.Files.v0.ExtractResult>('/fs/unarchive', {
			api: this.getBaseUrl(nodeInstance),
			version: 'v1',
			method: 'POST',
			params: { src: path, trg: '/', override, dry },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
		})
	}

	/**
	 * Modify a filesystem operation (dismiss or cancel)
	 *
	 * Uses v1 API endpoint.
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param opId - Operation ID (UUID)
	 * @param action - Action to perform
	 */
	public async modifyOperation(
		nodeInstance: string,
		nodeToken: string,
		opId: string,
		action: 'dismiss' | 'cancel',
	): Promise<void> {
		return this.client.request<void>(`/fs/ops/${action}`, {
			api: this.getBaseUrl(nodeInstance),
			version: 'v1',
			method: 'POST',
			params: { id: opId },
			headers: { Authorization: `Bearer ${nodeToken}` },
			skipAuth: true,
		})
	}
}
