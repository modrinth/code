import type { RequestContext } from '../types/request'

/**
 * Base configuration for features
 */
export interface FeatureConfig {
	/**
	 * Optional name for this feature (for debugging)
	 */
	name?: string

	/**
	 * Whether this feature is enabled
	 * @default true
	 */
	enabled?: boolean
}

/**
 * Abstract base class for request features
 *
 * Features are composable middleware that can intercept and modify requests.
 * They are executed in a chain, with each feature calling next() to continue the chain.
 */
export abstract class AbstractFeature {
	protected config: FeatureConfig

	constructor(config?: FeatureConfig) {
		this.config = {
			enabled: true,
			...config,
		}
	}

	/**
	 * Execute the feature logic
	 *
	 * @param next - Function to call the next feature in the chain (or the actual request)
	 * @param context - Full request context
	 * @returns Promise resolving to the response data
	 *
	 * @example
	 * ```typescript
	 * async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
	 *   // Do something before request
	 *   console.log('Before request:', context.url)
	 *
	 *   try {
	 *     const result = await next()
	 *
	 *     // Do something after successful request
	 *     console.log('Request succeeded')
	 *
	 *     return result
	 *   } catch (error) {
	 *     // Handle errors
	 *     console.error('Request failed:', error)
	 *     throw error
	 *   }
	 * }
	 * ```
	 */
	abstract execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T>

	/**
	 * Determine if this feature should apply to the given request
	 *
	 * By default, features apply if they are enabled.
	 * Override this to add custom logic (e.g., only apply to GET requests).
	 *
	 * @param context - Request context
	 * @returns true if the feature should execute, false to skip
	 */
	shouldApply(_context: RequestContext): boolean {
		return this.config.enabled !== false
	}

	/**
	 * Get the name of this feature (for debugging)
	 */
	get name(): string {
		return this.config.name ?? this.constructor.name
	}

	/**
	 * Check if this feature is enabled
	 */
	get enabled(): boolean {
		return this.config.enabled !== false
	}
}
