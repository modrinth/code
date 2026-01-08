import { FetchError } from 'ofetch'

import { ModrinthApiError } from '../core/errors'
import type { CircuitBreakerState, CircuitBreakerStorage } from '../features/circuit-breaker'
import type { ClientConfig } from '../types/client'
import type { RequestOptions } from '../types/request'
import type { UploadHandle, UploadRequestOptions } from '../types/upload'
import { GenericWebSocketClient } from './websocket-generic'
import { XHRUploadClient } from './xhr-upload-client'

/**
 * Circuit breaker storage using Nuxt's useState
 *
 * This provides cross-request persistence in SSR while also working in client-side.
 * State is shared between requests in the same Nuxt context.
 */
export class NuxtCircuitBreakerStorage implements CircuitBreakerStorage {
	private getState(): Map<string, CircuitBreakerState> {
		// @ts-expect-error - useState is provided by Nuxt runtime
		const state = useState<Map<string, CircuitBreakerState>>(
			'circuit-breaker-state',
			() => new Map(),
		)
		return state.value
	}

	get(key: string): CircuitBreakerState | undefined {
		return this.getState().get(key)
	}

	set(key: string, state: CircuitBreakerState): void {
		this.getState().set(key, state)
	}

	clear(key: string): void {
		this.getState().delete(key)
	}
}

/**
 * Nuxt-specific configuration
 */
export interface NuxtClientConfig extends ClientConfig {
	// TODO: do we want to provide this for tauri+base as well? its not used on app
	/**
	 * Rate limit key for server-side requests.
	 * This is injected as x-ratelimit-key header on server-side.
	 * Can be a string (for env var) or async function (for CF Secrets Store).
	 */
	rateLimitKey?: string | (() => Promise<string | undefined>)
}

/**
 * Nuxt platform client using Nuxt's $fetch
 *
 * This client is optimized for Nuxt applications and handles SSR/CSR automatically.
 *
 * Note: upload() is only available in browser context (CSR). It will throw during SSR.
 *
 * @example
 * ```typescript
 * // In a Nuxt composable
 * const config = useRuntimeConfig()
 * const auth = await useAuth()
 *
 * const client = new NuxtModrinthClient({
 *   userAgent: 'my-nuxt-app/1.0.0',
 *   rateLimitKey: import.meta.server ? config.rateLimitKey : undefined,
 *   features: [
 *     new AuthFeature({ token: () => auth.value.token })
 *   ]
 * })
 *
 * const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
 * ```
 */
export class NuxtModrinthClient extends XHRUploadClient {
	declare protected config: NuxtClientConfig
	private rateLimitKeyResolved: string | undefined
	private rateLimitKeyPromise: Promise<string | undefined> | undefined

	constructor(config: NuxtClientConfig) {
		super(config)

		Object.defineProperty(this.archon, 'sockets', {
			value: new GenericWebSocketClient(this),
			writable: false,
			enumerable: true,
			configurable: false,
		})
	}

	/**
	 * Resolve the rate limit key, handling both string and async function values.
	 * Results are cached for subsequent calls.
	 */
	private async resolveRateLimitKey(): Promise<string | undefined> {
		if (this.rateLimitKeyResolved !== undefined) {
			return this.rateLimitKeyResolved
		}

		const key = this.config.rateLimitKey
		if (typeof key === 'string') {
			this.rateLimitKeyResolved = key
		} else if (typeof key === 'function') {
			if (!this.rateLimitKeyPromise) {
				this.rateLimitKeyPromise = key()
			}
			this.rateLimitKeyResolved = await this.rateLimitKeyPromise
		}

		return this.rateLimitKeyResolved
	}

	/**
	 * Override request to resolve rate limit key before calling super.
	 * This allows async fetching of the key from CF Secrets Store.
	 */
	async request<T>(path: string, options: RequestOptions): Promise<T> {
		// @ts-expect-error - import.meta is provided by Nuxt
		if (import.meta.server) {
			await this.resolveRateLimitKey()
		}
		return super.request(path, options)
	}

	/**
	 * Upload a file with progress tracking
	 *
	 * Note: This method is only available in browser context (CSR).
	 * Calling during SSR will throw an error.
	 */
	upload<T = void>(path: string, options: UploadRequestOptions): UploadHandle<T> {
		// @ts-expect-error - import.meta is provided by Nuxt
		if (import.meta.server) {
			throw new ModrinthApiError('upload() is not supported during SSR')
		}
		return super.upload(path, options)
	}

	protected async executeRequest<T>(url: string, options: RequestOptions): Promise<T> {
		try {
			// @ts-expect-error - $fetch is provided by Nuxt runtime
			const response = await $fetch<T>(url, {
				method: options.method ?? 'GET',
				headers: options.headers,
				body: options.body,
				params: options.params,
				timeout: options.timeout,
				signal: options.signal,
			})

			return response
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

	protected buildDefaultHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			...super.buildDefaultHeaders(),
		}

		// Use the resolved key (populated by resolveRateLimitKey in request())
		// @ts-expect-error - import.meta is provided by Nuxt
		if (import.meta.server && this.rateLimitKeyResolved) {
			headers['x-ratelimit-key'] = this.rateLimitKeyResolved
		}

		return headers
	}
}
