import { $fetch, FetchError } from 'ofetch'

import { ModrinthApiError } from '../core/errors'
import type { ClientConfig } from '../types/client'
import type { RequestOptions } from '../types/request'
import { appendRequestParams, parseResponseErrorData, toFetchBody } from '../utils/fetch'
import { GenericSyncClient } from './sync-generic'
import { GenericWebSocketClient } from './websocket-generic'
import { XHRUploadClient } from './xhr-upload-client'

/**
 * Generic platform client using ofetch
 *
 * This client works in any JavaScript environment (Node.js, browser, workers, etc).
 *
 * @example
 * ```typescript
 * const client = new GenericModrinthClient({
 *   userAgent: 'my-app/1.0.0',
 *   features: [
 *     new AuthFeature({ token: async () => getOAuthToken() }),
 *     new RetryFeature({ maxAttempts: 3 })
 *   ]
 * })
 *
 * const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
 * ```
 */
export class GenericModrinthClient extends XHRUploadClient {
	constructor(config: ClientConfig) {
		super(config)

		Object.defineProperty(this.archon, 'sockets', {
			value: new GenericWebSocketClient(this),
			writable: false,
			enumerable: true,
			configurable: false,
		})
		Object.defineProperty(this.archon, 'sync', {
			value: new GenericSyncClient(this),
			writable: false,
			enumerable: true,
			configurable: false,
		})
	}

	protected async executeRequest<T>(url: string, options: RequestOptions): Promise<T> {
		try {
			const response = await $fetch<T>(url, {
				method: options.method ?? 'GET',
				headers: options.headers,
				body: options.body as BodyInit,
				params: options.params as Record<string, string>,
				timeout: options.timeout,
				signal: options.signal,
			})

			return response
		} catch (error) {
			// ofetch throws FetchError for HTTP errors
			throw this.normalizeError(error)
		}
	}

	protected async executeStreamRequest(
		url: string,
		options: RequestOptions,
	): Promise<ReadableStream<Uint8Array>> {
		try {
			const response = await fetch(appendRequestParams(url, options.params), {
				method: options.method ?? 'GET',
				headers: options.headers,
				body: toFetchBody(options.body),
				signal: options.signal,
			})

			if (!response.ok) {
				throw this.createNormalizedError(
					new Error(`HTTP ${response.status}: ${response.statusText}`),
					response.status,
					await parseResponseErrorData(response),
				)
			}

			if (!response.body) {
				throw new ModrinthApiError('Streaming response has no readable body', {
					statusCode: response.status,
				})
			}

			return response.body
		} catch (error) {
			throw this.normalizeError(error)
		}
	}

	protected normalizeError(error: unknown): ModrinthApiError {
		if (error instanceof FetchError) {
			return this.createNormalizedError(error, error.response?.status, error.data)
		}

		return super.normalizeError(error)
	}
}
