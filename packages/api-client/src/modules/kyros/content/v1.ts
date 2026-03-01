import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'

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
}
