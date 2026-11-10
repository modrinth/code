import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import { getNodeBaseUrl } from '../../../utils/node-url'
import type { Archon } from '../../archon/types'
import type { Kyros } from '../types'

export class KyrosContentV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_content_v1'
	}

	/** GET /v1/worlds/:world_id/content/embedded-icon */
	public async getEmbeddedAddonIcon(
		worldId: string,
		parentDirectory: 'mods' | 'plugins',
		filename: string,
	): Promise<Blob> {
		return this.client.request<Blob>(`/worlds/${worldId}/content/embedded-icon`, {
			api: '',
			version: 'v1',
			method: 'GET',
			responseType: 'blob',
			params: {
				parent_directory: parentDirectory,
				filename,
			},
			useNodeAuth: true,
		})
	}

	/**
	 * Upload addon files to a world via multipart form data
	 *
	 * @param worldId - World UUID
	 * @param files - Files to upload as addons
	 * @param options - Optional progress callback
	 * @returns UploadHandle with promise, onProgress, and cancel
	 * @deprecated Use `kyros.upload_sessions_v1` so cancellation can remove staged addon files before finalize.
	 */
	public uploadAddonFile(
		worldId: string,
		files: (File | Blob)[],
		options?: {
			onProgress?: (progress: UploadProgress) => void
		},
	): UploadHandle<void> {
		const formData = new FormData()
		for (const file of files) {
			formData.append('file', file, file instanceof File ? file.name : 'file')
		}

		return this.client.upload<void>(`/worlds/${worldId}/content/upload-addon-file`, {
			api: '',
			version: 'v1',
			formData,
			onProgress: options?.onProgress,
			useNodeAuth: true,
		})
	}

	/**
	 * POST /v1/worlds/:world_id/content/upload-modpack-file
	 *
	 * With `useExistingMrpack`, `file` must be `null`, and the mrpack left in the world by an
	 * earlier upload is installed instead.
	 */
	public uploadModpackFile(
		worldId: string,
		file: File | Blob | null,
		properties: Archon.Content.v1.PropertiesFields,
		options?: {
			softOverride?: boolean
			restoreClientFiles?: boolean
			noBackup?: boolean
			useExistingMrpack?: boolean
			onProgress?: (progress: UploadProgress) => void
			auth?: Archon.Servers.v0.JWTAuth
		},
	): UploadHandle<Kyros.Content.v1.UploadModpackFileResponse> {
		const formData = new FormData()
		if (file) formData.append('file', file, file instanceof File ? file.name : 'file')
		formData.append('properties', JSON.stringify(properties))

		const params: Record<string, string> = {}
		if (options?.softOverride !== undefined) params.soft_override = String(options.softOverride)
		if (options?.restoreClientFiles) params.restore_client_files = 'true'
		if (options?.noBackup) params.no_backup = 'true'
		if (options?.useExistingMrpack) params.use_existing_mrpack = 'true'

		return this.client.upload<Kyros.Content.v1.UploadModpackFileResponse>(
			`/worlds/${worldId}/content/upload-modpack-file`,
			{
				api: options?.auth ? getNodeBaseUrl(options.auth.url) : '',
				version: 'v1',
				formData,
				params: Object.keys(params).length ? params : undefined,
				onProgress: options?.onProgress,
				...(options?.auth
					? { headers: { Authorization: `Bearer ${options.auth.token}` }, skipAuth: true }
					: { useNodeAuth: true }),
			},
		)
	}
}
