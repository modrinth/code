import type { RequestOptions } from './request'

/**
 * Progress information for file uploads
 */
export interface UploadProgress {
	/** Bytes uploaded so far */
	loaded: number
	/** Total bytes to upload */
	total: number
	/** Progress as a decimal (0-1) */
	progress: number
}

/**
 * Options for upload requests (matches request() style)
 *
 * Extends RequestOptions but excludes body and method since those
 * are determined by the upload itself.
 */
export interface UploadRequestOptions extends Omit<RequestOptions, 'body' | 'method'> {
	/** File or Blob to upload */
	file: File | Blob
	/** Callback for progress updates */
	onProgress?: (progress: UploadProgress) => void
}

/**
 * Metadata attached to upload contexts
 *
 * Features can check `context.metadata?.isUpload` to detect uploads.
 */
export interface UploadMetadata extends Record<string, unknown> {
	isUpload: true
	file: File | Blob
	onProgress?: (progress: UploadProgress) => void
}

/**
 * Handle returned from upload operations
 *
 * Provides the upload promise, progress subscription, and cancellation.
 */
export interface UploadHandle<T> {
	/** Promise that resolves when upload completes */
	promise: Promise<T>
	/** Subscribe to progress updates (chainable) */
	onProgress: (callback: (progress: UploadProgress) => void) => UploadHandle<T>
	/** Cancel the upload */
	cancel: () => void
}
