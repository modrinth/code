import type { ModrinthApiError } from '../core/errors'
import type { ClientConfig } from '../types/client'
import type { RequestOptions } from '../types/request'
import { GenericWebSocketClient } from './websocket-generic'
import { XHRUploadClient } from './xhr-upload-client'

/**
 * Tauri-specific configuration
 * TODO: extend into interface if needed.
 */
export type TauriClientConfig = ClientConfig

/**
 * Extended error type with HTTP response metadata
 */
interface HttpError extends Error {
	statusCode?: number
	responseData?: unknown
}

/**
 * Tauri platform client using Tauri v2 HTTP plugin
 *
 * Extends XHRUploadClient to provide upload with progress tracking.
 *
 * @example
 * ```typescript
 * import { getVersion } from '@tauri-apps/api/app'
 *
 * const client = new TauriModrinthClient({
 *   userAgent: async () => `modrinth/theseus/${await getVersion()} (support@modrinth.com)`,
 *   features: [
 *     new AuthFeature({ token: async () => getOAuthToken() })
 *   ]
 * })
 *
 * const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
 * ```
 */
export class TauriModrinthClient extends XHRUploadClient {
	declare protected config: TauriClientConfig

	constructor(config: TauriClientConfig) {
		super(config)

		Object.defineProperty(this.archon, 'sockets', {
			value: new GenericWebSocketClient(this),
			writable: false,
			enumerable: true,
			configurable: false,
		})
	}

	protected async executeRequest<T>(url: string, options: RequestOptions): Promise<T> {
		try {
			// Dynamically import Tauri HTTP plugin
			// This allows the package to be used in non-Tauri environments
			const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http')

			let body: BodyInit | null | undefined = undefined
			if (options.body) {
				const raw = options.body
				if (
					typeof raw === 'object' &&
					!(raw instanceof FormData) &&
					!(raw instanceof URLSearchParams) &&
					!(raw instanceof Blob) &&
					!(raw instanceof ArrayBuffer) &&
					!ArrayBuffer.isView(raw as ArrayBufferView)
				) {
					body = JSON.stringify(raw)
				} else {
					body = raw as BodyInit
				}
			}

			let fullUrl = url
			if (options.params) {
				const filteredParams: Record<string, string> = {}
				for (const [key, value] of Object.entries(options.params)) {
					if (value !== undefined && value !== null) {
						filteredParams[key] = String(value)
					}
				}
				const queryString = new URLSearchParams(filteredParams).toString()
				if (queryString) {
					fullUrl = `${url}?${queryString}`
				}
			}

			const response = await tauriFetch(fullUrl, {
				method: options.method ?? 'GET',
				headers: options.headers,
				body,
			})

			if (!response.ok) {
				let responseData: unknown
				try {
					responseData = await response.json()
				} catch {
					responseData = undefined
				}

				const error = new Error(`HTTP ${response.status}: ${response.statusText}`) as HttpError

				error.statusCode = response.status
				error.responseData = responseData

				throw error
			}

			// Handle binary downloads (e.g. kyros fs files) before JSON parsing.
			const contentType = response.headers.get('content-type')?.toLowerCase() ?? ''
			if (fullUrl.includes('/fs/download')) {
				return (await response.blob()) as T
			}
			if (
				contentType.startsWith('image/') ||
				contentType.startsWith('audio/') ||
				contentType.startsWith('video/') ||
				contentType.includes('application/octet-stream')
			) {
				return (await response.blob()) as T
			}

			if (response.status === 204 || response.status === 205) {
				return undefined as T
			}

			if (contentType.includes('application/json') || contentType.includes('+json')) {
				return (await response.json()) as T
			}

			const text = await response.text()
			if (!text) {
				return undefined as T
			}

			try {
				return JSON.parse(text) as T
			} catch {
				return text as T
			}
		} catch (error) {
			throw this.normalizeError(error)
		}
	}

	protected normalizeError(error: unknown): ModrinthApiError {
		if (error instanceof Error) {
			const httpError = error as HttpError
			const statusCode = httpError.statusCode
			const responseData = httpError.responseData

			return this.createNormalizedError(error, statusCode, responseData)
		}

		return super.normalizeError(error)
	}
}
