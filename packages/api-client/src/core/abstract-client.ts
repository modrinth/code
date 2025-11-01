import type { ClientConfig } from '../types/client'
import type { RequestContext, RequestOptions } from '../types/request'
import type { AbstractFeature } from './abstract-feature'
import { ModrinthApiError } from './errors'

/**
 * Abstract base client for Modrinth APIs
 *
 * This class provides the core functionality for making API requests with features.
 * Platform-specific implementations (Nuxt, Tauri, generic) should extend this class
 * and implement the executeRequest method.
 */
export abstract class AbstractModrinthClient {
	protected config: ClientConfig
	protected features: AbstractFeature[]

	constructor(config: ClientConfig = {}) {
		this.config = {
			timeout: 10000,
			labrinthBaseUrl: 'https://api.modrinth.com',
			archonBaseUrl: 'https://archon.modrinth.com',
			...config,
		}
		this.features = config.features ?? []
	}

	/**
	 * Make a request to the API
	 *
	 * This is the main public method that applications use to make requests.
	 * It handles URL building, feature execution, and error normalization.
	 *
	 * @param path - API path (e.g., '/project/sodium')
	 * @param options - Request options
	 * @returns Promise resolving to the response data
	 * @throws {ModrinthApiError} When the request fails or features throw errors
	 */
	async request<T>(path: string, options: RequestOptions): Promise<T> {
		let baseUrl: string
		if (options.api === 'labrinth') {
			baseUrl = this.config.labrinthBaseUrl!
		} else if (options.api === 'archon') {
			baseUrl = this.config.archonBaseUrl!
		} else {
			baseUrl = options.api
		}

		const url = this.buildUrl(path, baseUrl, options.version)

		// Merge options with defaults
		const mergedOptions: RequestOptions = {
			method: 'GET',
			timeout: this.config.timeout,
			...options,
			headers: {
				...this.buildDefaultHeaders(),
				...options.headers,
			},
		}

		const context = this.buildContext(url, path, mergedOptions)

		try {
			const result = await this.executeFeatureChain<T>(context)

			await this.config.hooks?.onResponse?.(result, context)

			return result
		} catch (error) {
			const apiError = this.normalizeError(error, context)
			await this.config.hooks?.onError?.(apiError, context)

			throw apiError
		}
	}

	/**
	 * Execute the feature chain and the actual request
	 *
	 * Features are executed in order, with each feature calling next() to continue.
	 * The last "feature" in the chain is the actual request execution.
	 */
	protected async executeFeatureChain<T>(context: RequestContext): Promise<T> {
		// Filter to only features that should apply
		const applicableFeatures = this.features.filter((feature) => feature.shouldApply(context))

		// Build the feature chain
		// We work backwards from the actual request, wrapping each feature around the previous
		let index = applicableFeatures.length

		const next = async (): Promise<T> => {
			index--

			if (index >= 0) {
				// Execute the next feature in the chain
				const feature = applicableFeatures[index]
				return feature.execute(next, context)
			} else {
				// We've reached the end of the chain, execute the actual request
				await this.config.hooks?.onRequest?.(context)
				return this.executeRequest<T>(context.url, context.options)
			}
		}

		return next()
	}

	/**
	 * Build the full URL for a request
	 *
	 * This handles:
	 * - Base URL
	 * - API versioning (v2, v3, internal)
	 * - Path normalization
	 */
	protected buildUrl(path: string, baseUrl: string, version: number | 'internal'): string {
		// Remove trailing slash from base URL
		const base = baseUrl.replace(/\/$/, '')

		// Build version path
		let versionPath = ''
		if (version === 'internal') {
			versionPath = '/_internal'
		} else if (typeof version === 'number') {
			versionPath = `/v${version}`
		}

		// Ensure path starts with /
		const cleanPath = path.startsWith('/') ? path : `/${path}`

		return `${base}${versionPath}${cleanPath}`
	}

	/**
	 * Build the request context
	 */
	protected buildContext(url: string, path: string, options: RequestOptions): RequestContext {
		return {
			url,
			path,
			options,
			attempt: 1,
			startTime: Date.now(),
		}
	}

	/**
	 * Build default headers for all requests
	 *
	 * Subclasses can override this to add platform-specific headers
	 * (e.g., Nuxt rate limit key)
	 */
	protected buildDefaultHeaders(): Record<string, string> {
		return {
			'Content-Type': 'application/json',
			'User-Agent': this.config.userAgent,
			...this.config.headers,
		}
	}

	/**
	 * Execute the actual HTTP request
	 *
	 * This must be implemented by platform-specific clients.
	 * Platform implementations should use their native fetch mechanism:
	 * - Generic: ofetch
	 * - Nuxt: $fetch
	 * - Tauri: @tauri-apps/plugin-http
	 *
	 * @param url - Full URL to request
	 * @param options - Request options
	 * @returns Promise resolving to the response data
	 * @throws {Error} Platform-specific errors that will be normalized by normalizeError()
	 */
	protected abstract executeRequest<T>(url: string, options: RequestOptions): Promise<T>

	/**
	 * Normalize an error into a ModrinthApiError
	 *
	 * Platform implementations should override this to handle platform-specific errors
	 * (e.g., FetchError from ofetch, Tauri HTTP errors)
	 */
	protected normalizeError(error: unknown, context?: RequestContext): ModrinthApiError {
		if (error instanceof ModrinthApiError) {
			return error
		}

		return ModrinthApiError.fromUnknown(error, context?.path)
	}

	/**
	 * Add a feature to this client
	 *
	 * Features are executed in the order they are added.
	 *
	 * @example
	 * ```typescript
	 * const client = new GenericModrinthClient()
	 * client.addFeature(new AuthFeature({ token: 'mrp_...' }))
	 * client.addFeature(new RetryFeature({ maxAttempts: 3 }))
	 * ```
	 */
	addFeature(feature: AbstractFeature): this {
		this.features.push(feature)
		return this
	}

	/**
	 * Remove a feature from this client
	 *
	 * @example
	 * ```typescript
	 * const retryFeature = new RetryFeature({ maxAttempts: 3 })
	 * client.addFeature(retryFeature)
	 * // Later, remove it
	 * client.removeFeature(retryFeature)
	 * ```
	 */
	removeFeature(feature: AbstractFeature): this {
		const index = this.features.indexOf(feature)
		if (index !== -1) {
			this.features.splice(index, 1)
		}
		return this
	}

	/**
	 * Get all features on this client
	 */
	getFeatures(): AbstractFeature[] {
		return [...this.features]
	}
}
