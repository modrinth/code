import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import { ModrinthApiError } from '../core/errors'
import type { RequestContext } from '../types/request'

/**
 * Circuit breaker state
 */
export type CircuitBreakerState = {
	/**
	 * Number of consecutive failures
	 */
	failures: number

	/**
	 * Timestamp of last failure
	 */
	lastFailure: number
}

/**
 * Circuit breaker storage interface
 */
export interface CircuitBreakerStorage {
	/**
	 * Get circuit breaker state for a key
	 */
	get(key: string): CircuitBreakerState | undefined

	/**
	 * Set circuit breaker state for a key
	 */
	set(key: string, state: CircuitBreakerState): void

	/**
	 * Clear circuit breaker state for a key
	 */
	clear?(key: string): void
}

/**
 * Circuit breaker feature configuration
 */
export interface CircuitBreakerConfig extends FeatureConfig {
	/**
	 * Maximum number of consecutive failures before opening circuit
	 * @default 3
	 */
	maxFailures?: number

	/**
	 * Time in milliseconds before circuit resets after opening
	 * @default 30000
	 */
	resetTimeout?: number

	/**
	 * HTTP status codes that count as failures
	 * @default [500, 502, 503, 504]
	 */
	failureStatusCodes?: number[]

	/**
	 * Storage implementation for circuit state
	 * If not provided, uses in-memory Map
	 */
	storage?: CircuitBreakerStorage

	/**
	 * Function to generate circuit key from request context
	 * By default, uses the base path (without query params)
	 */
	getCircuitKey?: (url: string, method: string) => string
}

/**
 * In-memory storage for circuit breaker state
 */
export class InMemoryCircuitBreakerStorage implements CircuitBreakerStorage {
	private state = new Map<string, CircuitBreakerState>()

	get(key: string): CircuitBreakerState | undefined {
		return this.state.get(key)
	}

	set(key: string, state: CircuitBreakerState): void {
		this.state.set(key, state)
	}

	clear(key: string): void {
		this.state.delete(key)
	}
}

/**
 * Circuit breaker feature
 *
 * Prevents requests to failing endpoints by "opening the circuit" after
 * a threshold of consecutive failures. The circuit automatically resets
 * after a timeout period.
 *
 * This implements the circuit breaker pattern to prevent cascading failures
 * and give failing services time to recover.
 *
 * @example
 * ```typescript
 * const circuitBreaker = new CircuitBreakerFeature({
 *   maxFailures: 3,
 *   resetTimeout: 30000, // 30 seconds
 *   failureStatusCodes: [500, 502, 503, 504]
 * })
 * ```
 */
export class CircuitBreakerFeature extends AbstractFeature {
	protected declare config: Required<CircuitBreakerConfig>
	private storage: CircuitBreakerStorage

	constructor(config?: CircuitBreakerConfig) {
		super(config)

		this.config = {
			enabled: true,
			name: 'circuit-breaker',
			maxFailures: 3,
			resetTimeout: 30000,
			failureStatusCodes: [500, 502, 503, 504],
			...config,
		} as Required<CircuitBreakerConfig>

		// Use provided storage or default to in-memory
		this.storage = config?.storage ?? new InMemoryCircuitBreakerStorage()
	}

	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		const circuitKey = this.getCircuitKey(context)

		if (this.isCircuitOpen(circuitKey)) {
			throw new ModrinthApiError('Circuit breaker open - too many recent failures', {
				statusCode: 503,
				context: context.path,
			})
		}

		try {
			const result = await next()

			this.recordSuccess(circuitKey)

			return result
		} catch (error) {
			if (this.isFailureError(error)) {
				this.recordFailure(circuitKey)
			}

			throw error
		}
	}

	shouldApply(context: RequestContext): boolean {
		if (context.options.circuitBreaker === false) {
			return false
		}

		return super.shouldApply(context)
	}

	/**
	 * Get the circuit key for a request
	 *
	 * By default, uses the path and method to identify unique circuits
	 */
	private getCircuitKey(context: RequestContext): string {
		if (this.config.getCircuitKey) {
			return this.config.getCircuitKey(context.url, context.options.method ?? 'GET')
		}

		// Default: use method + path (without query params)
		const method = context.options.method ?? 'GET'
		const pathWithoutQuery = context.path.split('?')[0]

		return `${method}_${pathWithoutQuery}`
	}

	/**
	 * Check if the circuit is open for a given key
	 */
	private isCircuitOpen(key: string): boolean {
		const state = this.storage.get(key)

		if (!state) {
			return false
		}

		const now = Date.now()
		const timeSinceLastFailure = now - state.lastFailure

		if (timeSinceLastFailure > this.config.resetTimeout) {
			this.storage.clear?.(key)
			return false
		}

		return state.failures >= this.config.maxFailures
	}

	/**
	 * Record a successful request
	 */
	private recordSuccess(key: string): void {
		this.storage.clear?.(key)
	}

	/**
	 * Record a failed request
	 */
	private recordFailure(key: string): void {
		const now = Date.now()
		const state = this.storage.get(key)

		if (!state) {
			// First failure
			this.storage.set(key, {
				failures: 1,
				lastFailure: now,
			})
		} else {
			// Subsequent failure
			this.storage.set(key, {
				failures: state.failures + 1,
				lastFailure: now,
			})
		}
	}

	/**
	 * Determine if an error should count as a circuit failure
	 */
	private isFailureError(error: unknown): boolean {
		if (error instanceof ModrinthApiError && error.statusCode) {
			return this.config.failureStatusCodes.includes(error.statusCode)
		}

		return false
	}

	/**
	 * Get current circuit state for debugging
	 *
	 * @example
	 * ```typescript
	 * const state = circuitBreaker.getCircuitState('GET_/v2/project/sodium')
	 * console.log(`Failures: ${state?.failures}, Last failure: ${state?.lastFailure}`)
	 * ```
	 */
	getCircuitState(key: string): CircuitBreakerState | undefined {
		return this.storage.get(key)
	}

	/**
	 * Manually reset a circuit
	 *
	 * @example
	 * ```typescript
	 * // Reset circuit after manual intervention
	 * circuitBreaker.resetCircuit('GET_/v2/project/sodium')
	 * ```
	 */
	resetCircuit(key: string): void {
		this.storage.clear?.(key)
	}
}
