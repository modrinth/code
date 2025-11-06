import { $fetch, FetchError } from 'ofetch'

import { AbstractModrinthClient } from '../core/abstract-client'
import type { ModrinthApiError } from '../core/errors'
import type { RequestOptions } from '../types/request'

/**
 * Generic platform client using ofetch
 *
 * This client works in any JavaScript environment (Node.js, browser, workers).
 *
 * @example
 * ```typescript
 * const client = new GenericModrinthClient({
 *   userAgent: 'my-app/1.0.0',
 *   features: [
 *     new AuthFeature({ token: 'mrp_...' }),
 *     new RetryFeature({ maxAttempts: 3 })
 *   ]
 * })
 *
 * const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
 * ```
 */
export class GenericModrinthClient extends AbstractModrinthClient {
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

	protected normalizeError(error: unknown): ModrinthApiError {
		if (error instanceof FetchError) {
			return this.createNormalizedError(error, error.response?.status, error.data)
		}

		return super.normalizeError(error)
	}
}
