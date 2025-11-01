import { AbstractModrinthClient } from '../core/abstract-client'
import { ModrinthApiError } from '../core/errors'
import type { ClientConfig } from '../types/client'
import type { RequestOptions } from '../types/request'

/**
 * Tauri-specific configuration
 */
export interface TauriClientConfig extends ClientConfig {
	// No additional Tauri-specific config currently
}

/**
 * Tauri platform client using Tauri v2 HTTP plugin
 *
 * This client is optimized for Tauri applications and uses the Tauri HTTP plugin
 * which provides native HTTP requests with proper CORS handling and system integration.
 *
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
				throw new Error(`HTTP ${response.status}: ${response.statusText}`)
			}

			const data = await response.json()
			return data as T
		} catch (error) {
			throw this.normalizeError(error)
		}
	}

	protected normalizeError(error: unknown): ModrinthApiError {
		if (error instanceof Error) {
			// Try to extract status code from error message
			const statusMatch = error.message.match(/HTTP (\d+)/)
			const statusCode = statusMatch ? parseInt(statusMatch[1]) : undefined

			return new ModrinthApiError(error.message, {
				statusCode,
				originalError: error,
			})
		}

		return super.normalizeError(error)
	}
}
