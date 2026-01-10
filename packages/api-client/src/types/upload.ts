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
 * Base options for upload requests
 */
interface BaseUploadRequestOptions extends Omit<RequestOptions, 'body' | 'method'> {
	/** Callback for progress updates */
	onProgress?: (progress: UploadProgress) => void
}

/**
 * Options for single file upload requests
 */
export interface FileUploadRequestOptions extends BaseUploadRequestOptions {
	/** File or Blob to upload */
	file: File | Blob
	formData?: never
}

/**
 * Options for FormData upload requests
 *
 * Used for multipart uploads (e.g., version file uploads) that need
 * to send metadata alongside files.
 */
export interface FormDataUploadRequestOptions extends BaseUploadRequestOptions {
	/** FormData containing files and metadata */
	formData: FormData
	file?: never
}

/**
 * Options for upload requests - either a single file or FormData
 */
export type UploadRequestOptions = FileUploadRequestOptions | FormDataUploadRequestOptions

/**
 * Metadata attached to file upload contexts
 *
 * Features can check `context.metadata?.isUpload` to detect uploads.
 */
export interface FileUploadMetadata extends Record<string, unknown> {
	isUpload: true
	file: File | Blob
	formData?: never
	onProgress?: (progress: UploadProgress) => void
}

/**
 * Metadata attached to FormData upload contexts
 */
export interface FormDataUploadMetadata extends Record<string, unknown> {
	isUpload: true
	formData: FormData
	file?: never
	onProgress?: (progress: UploadProgress) => void
}

/**
 * Metadata attached to upload contexts - either file or FormData
 */
export type UploadMetadata = FileUploadMetadata | FormDataUploadMetadata

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
