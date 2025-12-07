import { AbstractModrinthClient } from '../core/abstract-client'
import type { ModrinthApiError } from '../core/errors'
import type { ClientConfig } from '../types/client'
import type { RequestOptions } from '../types/request'
import { GenericWebSocketClient } from './websocket-generic'

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

 * @example
 * ```typescript
 * import { getVersion } from '@tauri-apps/api/app'
 *
 * const version = await getVersion()
 * const client = new TauriModrinthClient({
 *   userAgent: `modrinth/theseus/${version} (support@modrinth.com)`,
 *   features: [
 *     new AuthFeature({ token: 'mrp_...' })
 *   ]
 * })
 *
 * const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
 * ```
 */
export class TauriModrinthClient extends AbstractModrinthClient {
	protected declare config: TauriClientConfig

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
				if (typeof options.body === 'object' && !(options.body instanceof FormData)) {
					body = JSON.stringify(options.body)
				} else {
					body = options.body as BodyInit
				}
			}

			let fullUrl = url
			if (options.params) {
				const queryParams = new URLSearchParams(options.params as Record<string, string>).toString()
				fullUrl = `${url}?${queryParams}`
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

			const data = await response.json()
			return data as T
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
