import type { UploadHandle, UploadRequestOptions } from '../types/upload'

/**
 * Abstract base class defining upload capability
 *
 * All clients that support file uploads must extend this class.
 * Platform-specific implementations should provide the actual upload mechanism
 * (e.g., XHR for browser environments).
 *
 * Upload goes through the feature chain (auth, retry, circuit-breaker, etc.)
 * just like regular requests.
 */
export abstract class AbstractUploadClient {
	/**
	 * Upload a file with progress tracking
	 *
	 * Features (auth, retry, etc.) are applied to uploads.
	 * Retry is disabled by default to prevent re-uploading large files.
	 *
	 * @param path - API path (e.g., '/fs/create')
	 * @param options - Upload options including file, api, version
	 * @returns UploadHandle with promise, onProgress chain, and cancel method
	 */
	abstract upload<T = void>(path: string, options: UploadRequestOptions): UploadHandle<T>
}
