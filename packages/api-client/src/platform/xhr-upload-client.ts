import { AbstractModrinthClient } from '../core/abstract-client'
import { ModrinthApiError } from '../core/errors'
import type { RequestContext } from '../types/request'
import type {
	UploadHandle,
	UploadMetadata,
	UploadProgress,
	UploadRequestOptions,
} from '../types/upload'

/**
 * Abstract client with XHR-based upload implementation
 *
 * Provides upload() with progress tracking for browser environments.
 * Uses XMLHttpRequest because fetch doesn't support upload progress in Firefox/Safari.
 *
 * Uploads go through the feature chain (auth, retry, circuit-breaker, etc.)
 * just like regular requests.
 *
 * Platform-specific clients should extend this instead of AbstractModrinthClient
 * to inherit the XHR upload implementation.
 */
export abstract class XHRUploadClient extends AbstractModrinthClient {
	/**
	 * Upload a file with progress tracking
	 *
	 * Goes through the feature chain (auth, retry, etc.)
	 * Retry is disabled by default to prevent re-uploading large files.
	 */
	upload<T = void>(path: string, options: UploadRequestOptions): UploadHandle<T> {
		// Build URL like request() does
		let baseUrl: string
		if (options.api === 'labrinth') {
			baseUrl = this.config.labrinthBaseUrl!
		} else if (options.api === 'archon') {
			baseUrl = this.config.archonBaseUrl!
		} else {
			baseUrl = options.api
		}

		const url = this.buildUrl(path, baseUrl, options.version)

		// Merge with defaults - retry defaults to false for uploads
		const mergedOptions: UploadRequestOptions = {
			retry: false, // Default: don't retry uploads
			...options,
			headers: {
				...this.buildDefaultHeaders(),
				'Content-Type': 'application/octet-stream',
				...options.headers,
			},
		}

		const context = this.buildUploadContext(url, path, mergedOptions)

		// Setup progress callbacks and abort controller
		const progressCallbacks: Array<(p: UploadProgress) => void> = []
		if (mergedOptions.onProgress) {
			progressCallbacks.push(mergedOptions.onProgress)
		}
		const abortController = new AbortController()

		// Link external signal if provided
		if (mergedOptions.signal) {
			mergedOptions.signal.addEventListener('abort', () => abortController.abort())
		}

		// Build the handle - promise goes through feature chain
		const handle: UploadHandle<T> = {
			promise: this.executeUploadFeatureChain<T>(context, progressCallbacks, abortController)
				.then(async (result) => {
					await this.config.hooks?.onResponse?.(result, context)
					return result
				})
				.catch(async (error) => {
					const apiError = this.normalizeError(error, context)
					await this.config.hooks?.onError?.(apiError, context)
					throw apiError
				}),
			onProgress: (callback) => {
				progressCallbacks.push(callback)
				return handle
			},
			cancel: () => abortController.abort(),
		}

		return handle
	}

	/**
	 * Execute the actual XHR upload (called at end of feature chain)
	 */
	protected executeXHRUpload<T>(
		context: RequestContext,
		progressCallbacks: Array<(p: UploadProgress) => void>,
		abortController: AbortController,
	): Promise<T> {
		return new Promise<T>((resolve, reject) => {
			const xhr = new XMLHttpRequest()
			const metadata = context.metadata as UploadMetadata

			xhr.upload.addEventListener('progress', (e) => {
				if (e.lengthComputable) {
					const progress: UploadProgress = {
						loaded: e.loaded,
						total: e.total,
						progress: e.loaded / e.total,
					}
					progressCallbacks.forEach((cb) => cb(progress))
				}
			})

			xhr.addEventListener('load', () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					try {
						resolve(xhr.response ? JSON.parse(xhr.response) : (undefined as T))
					} catch {
						resolve(undefined as T)
					}
				} else {
					reject(this.createUploadError(xhr))
				}
			})

			xhr.addEventListener('error', () => reject(new ModrinthApiError('Upload failed')))
			xhr.addEventListener('abort', () => reject(new ModrinthApiError('Upload cancelled')))

			// Build URL with params (unlike $fetch, XHR doesn't handle params automatically)
			let url = context.url
			if (context.options.params) {
				const queryString = new URLSearchParams(
					Object.entries(context.options.params).map(([k, v]) => [k, String(v)]),
				).toString()
				url += (url.includes('?') ? '&' : '?') + queryString
			}

			xhr.open('POST', url)

			// Apply headers from context (features may have modified them)
			for (const [key, value] of Object.entries(context.options.headers ?? {})) {
				xhr.setRequestHeader(key, value)
			}

			xhr.send(metadata.file)
			abortController.signal.addEventListener('abort', () => xhr.abort())
		})
	}

	protected createUploadError(xhr: XMLHttpRequest): ModrinthApiError {
		let responseData: unknown
		try {
			responseData = xhr.response ? JSON.parse(xhr.response) : undefined
		} catch {
			responseData = xhr.responseText
		}
		return this.createNormalizedError(
			new Error(`Upload failed with status ${xhr.status}`),
			xhr.status,
			responseData,
		)
	}
}
