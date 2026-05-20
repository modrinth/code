import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import type { Archon } from '../../archon/types'

export class KyrosContentV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_content_v1'
	}

	/**
	 * Upload addon files to a world via multipart form data
	 *
	 * @param worldId - World UUID
	 * @param files - Files to upload as addons
	 * @param options - Optional progress callback
	 * @returns UploadHandle with promise, onProgress, and cancel
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

	/** POST /v1/worlds/:world_id/content/upload-modpack-file */
	public uploadModpackFile(
		worldId: string,
		file: File | Blob,
		properties: Archon.Content.v1.PropertiesFields,
		options?: {
			softOverride?: boolean
			onProgress?: (progress: UploadProgress) => void
		},
	): UploadHandle<void> {
		const formData = new FormData()
		formData.append('file', file, file instanceof File ? file.name : 'file')
		formData.append('properties', JSON.stringify(properties))

		return this.client.upload<void>(`/worlds/${worldId}/content/upload-modpack-file`, {
			api: '',
			version: 'v1',
			formData,
			params:
				options?.softOverride !== undefined
					? { soft_override: String(options.softOverride) }
					: undefined,
			onProgress: options?.onProgress,
			useNodeAuth: true,
		})
	}
}
