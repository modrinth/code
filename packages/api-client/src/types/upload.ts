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
 * A file on the local filesystem, which the platform uploads straight from disk rather than
 * reading it into memory. Requires a `localFileUploader` in the client config.
 */
export interface LocalFile {
	kind: 'local'
	path: string
	name: string
	size: number
}

/**
 * A file that can be uploaded, either held by the browser or on the local filesystem
 */
export type UploadFile = File | Blob | LocalFile

export function isLocalFile(file: unknown): file is LocalFile {
	return typeof file === 'object' && file !== null && (file as LocalFile).kind === 'local'
}

/**
 * A local file upload, after the feature chain has applied auth and other headers
 */
export interface LocalFileUploadRequest {
	url: string
	headers: Record<string, string>
	file: LocalFile
	/** Sends the file as this multipart form field rather than as the raw body */
	multipartField?: string
	signal: AbortSignal
	onProgress: (progress: UploadProgress) => void
}

/**
 * The response to a local file upload, whatever its status
 */
export interface LocalFileUploadResponse {
	status: number
	body: string
}

/**
 * Sends local file uploads for the platform, such as through a native HTTP client
 */
export type LocalFileUploader = (
	request: LocalFileUploadRequest,
) => Promise<LocalFileUploadResponse>

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
	/** File, Blob, or local file to upload */
	file: UploadFile
	/** Sends the file as this multipart form field rather than as the raw body */
	multipartField?: string
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
	file: UploadFile
	multipartField?: string
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

/**
 * State of a batch file upload operation
 */
export interface UploadState {
	isUploading: boolean
	currentFileName: string | null
	currentFileProgress: number
	uploadedBytes: number
	totalBytes: number
	completedFiles: number
	totalFiles: number
}
