import type { InferredClientModules } from '../modules'
import { buildModuleStructure } from '../modules'
import type { ClientConfig } from '../types/client'
import type { RequestContext, RequestOptions } from '../types/request'
import type { AbstractFeature } from './abstract-feature'
import type { AbstractModule } from './abstract-module'
import type { AbstractWebSocketClient } from './abstract-websocket'
import { ModrinthApiError, ModrinthServerError } from './errors'

/**
 * Abstract base client for Modrinth APIs
 */
export abstract class AbstractModrinthClient {
	protected config: ClientConfig
	protected features: AbstractFeature[]

	/**
	 * Maps full module ID (e.g., 'labrinth_projects_v2') to instantiated module
	 */
	private _moduleInstances: Map<string, AbstractModule> = new Map()

	/**
	 * Maps API name (e.g., 'labrinth') to namespace object
	 */
	private _moduleNamespaces: Map<string, Record<string, AbstractModule>> = new Map()

	public readonly labrinth!: InferredClientModules['labrinth']
	public readonly archon!: InferredClientModules['archon'] & { sockets: AbstractWebSocketClient }
	public readonly kyros!: InferredClientModules['kyros']
	public readonly iso3166!: InferredClientModules['iso3166']

	constructor(config: ClientConfig) {
		this.config = {
			timeout: 10000,
			labrinthBaseUrl: 'https://api.modrinth.com',
			archonBaseUrl: 'https://archon.modrinth.com',
			...config,
		}
		this.features = config.features ?? []
		this.initializeModules()
	}

	/**
	 * This creates the nested API structure (e.g., client.labrinth.projects_v2)
	 * but doesn't instantiate modules until first access
	 *
	 * Module IDs in the registry are validated at runtime to ensure they match
	 * what the module declares via getModuleID().
	 */
	private initializeModules(): void {
		const structure = buildModuleStructure()

		for (const [api, modules] of Object.entries(structure)) {
			const namespaceObj: Record<string, AbstractModule> = {}

			// Define lazy getters for each module
			for (const [moduleName, ModuleConstructor] of Object.entries(modules)) {
				const fullModuleId = `${api}_${moduleName}`

				Object.defineProperty(namespaceObj, moduleName, {
					get: () => {
						// Lazy instantiation
						if (!this._moduleInstances.has(fullModuleId)) {
							const instance = new ModuleConstructor(this)

							// Validate the module ID matches what we expect
							const declaredId = instance.getModuleID()
							if (declaredId !== fullModuleId) {
								throw new Error(
									`Module ID mismatch: registry expects "${fullModuleId}" but module declares "${declaredId}"`,
								)
							}

							this._moduleInstances.set(fullModuleId, instance)
						}
						return this._moduleInstances.get(fullModuleId)!
					},
					enumerable: true,
					configurable: false,
				})
			}

			// Assign namespace to client (e.g., this.labrinth = namespaceObj)
			// defineProperty bypasses readonly restriction
			Object.defineProperty(this, api, {
				value: namespaceObj,
				writable: false,
				enumerable: true,
				configurable: false,
			})

			this._moduleNamespaces.set(api, namespaceObj)
		}
	}

	/**
	 * Make a request to the API
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

		const headers = mergedOptions.headers
		if (headers && 'Content-Type' in headers && headers['Content-Type'] === '') {
			delete headers['Content-Type']
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
	 */
	protected buildUrl(path: string, baseUrl: string, version: number | 'internal' | string): string {
		// Remove trailing slash from base URL
		const base = baseUrl.replace(/\/$/, '')

		// Build version path
		let versionPath = ''
		if (version === 'internal') {
			versionPath = '/_internal'
		} else if (typeof version === 'number') {
			versionPath = `/v${version}`
		} else if (typeof version === 'string') {
			// Custom version string (e.g., 'v0', 'modrinth/v0')
			versionPath = `/${version}`
		}

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
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			...this.config.headers,
		}

		if (this.config.userAgent) {
			headers['User-Agent'] = this.config.userAgent
		}

		return headers
	}

	/**
	 * Execute the actual HTTP request
	 *
	 * This must be implemented by platform-specific clients.
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
	 * Helper to create a normalized error from extracted status code and response data
	 */
	protected createNormalizedError(
		error: Error,
		statusCode: number | undefined,
		responseData: unknown,
	): ModrinthApiError {
		if (statusCode && responseData) {
			return ModrinthServerError.fromResponse(statusCode, responseData)
		}

		return new ModrinthApiError(error.message, {
			statusCode,
			originalError: error,
			responseData,
		})
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
