import { type LocalFileUploader, ModrinthApiError } from '@modrinth/api-client'
import { Channel } from '@tauri-apps/api/core'

import {
	cancelLocalUpload,
	type LocalUploadProgress,
	uploadLocal,
} from '@/platform/app-lib/files/commands'

/** Uploads local files from the Rust side, streaming them from disk. */
export function createLocalFileUploader(): LocalFileUploader {
	return async ({ url, headers, file, multipartField, signal, onProgress }) => {
		const uploadId = crypto.randomUUID()
		const cancel = () => void cancelLocalUpload(uploadId).catch(() => undefined)
		signal.addEventListener('abort', cancel)
		try {
			return await uploadLocal(
				uploadId,
				{ url, headers, path: file.path, fileName: file.name, multipartField },
				new Channel<LocalUploadProgress>(({ loaded, total }) =>
					onProgress({ loaded, total, progress: total > 0 ? loaded / total : 1 }),
				),
			)
		} catch (error) {
			throw new ModrinthApiError((error as { message?: string }).message ?? String(error))
		} finally {
			signal.removeEventListener('abort', cancel)
		}
	}
}
