import { AbstractModule } from '../../../core/abstract-module'

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
	 * Upload a file to a server's filesystem
	 *
	 * @param nodeInstance - Node instance URL
	 * @param nodeToken - JWT token from getFilesystemAuth
	 * @param path - Destination path (e.g., "/server-icon.png")
	 * @param file - File to upload
	 */
	public async uploadFile(
		nodeInstance: string,
		nodeToken: string,
		path: string,
		file: File,
	): Promise<void> {
		return this.client.request<void>(`/fs/create`, {
			api: `https://${nodeInstance.replace('v0/fs', '')}`,
			method: 'POST',
			version: 'v0',
			params: { path, type: 'file' },
			headers: {
				Authorization: `Bearer ${nodeToken}`,
				'Content-Type': 'application/octet-stream',
			},
			body: file,
		})
	}
}
