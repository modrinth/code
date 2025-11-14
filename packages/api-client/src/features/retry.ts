import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import { ModrinthApiError } from '../core/errors'
import type { RequestContext } from '../types/request'

/**
 * Backoff strategy for retries
 */
export type BackoffStrategy = 'exponential' | 'linear' | 'constant'

/**
 * Retry feature configuration
 */
export interface RetryConfig extends FeatureConfig {
	/**
	 * Maximum number of retry attempts
	 * @default 3
	 */
	maxAttempts?: number

	/**
	 * Backoff strategy to use
	 * @default 'exponential'
	 */
	backoffStrategy?: BackoffStrategy

	/**
	 * Initial delay in milliseconds before first retry
	 * @default 1000
	 */
	initialDelay?: number

	/**
	 * Maximum delay in milliseconds between retries
	 * @default 15000
	 */
	maxDelay?: number

	/**
	 * HTTP status codes that should trigger a retry
	 * @default [408, 429, 500, 502, 503, 504]
	 */
	retryableStatusCodes?: number[]

	/**
	 * Whether to retry on network errors (connection refused, timeout, etc.)
	 * @default true
	 */
	retryOnNetworkError?: boolean

	/**
	 * Custom function to determine if an error should be retried
	 */
	shouldRetry?: (error: unknown, attempt: number) => boolean
}

/**
 * Retry feature
 *
 * Automatically retries failed requests with configurable backoff strategy.
 * Only retries errors that are likely to succeed on retry (e.g., timeout, 5xx errors).
 *
 * @example
 * ```typescript
 * const retry = new RetryFeature({
 *   maxAttempts: 3,
 *   backoffStrategy: 'exponential',
 *   initialDelay: 1000,
 *   maxDelay: 15000
 * })
 * ```
 */
export class RetryFeature extends AbstractFeature {
	protected declare config: Required<RetryConfig>

	constructor(config?: RetryConfig) {
		super(config)

		this.config = {
			enabled: true,
			name: 'retry',
			maxAttempts: 3,
			backoffStrategy: 'exponential',
			initialDelay: 1000,
			maxDelay: 15000,
			retryableStatusCodes: [408, 429, 500, 502, 503, 504],
			retryOnNetworkError: true,
			...config,
		} as Required<RetryConfig>
	}

	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		let lastError: Error | null = null
		const maxAttempts = this.getMaxAttempts(context)

		for (let attempt = 1; attempt <= maxAttempts; attempt++) {
			context.attempt = attempt

			try {
				const result = await next()
				return result
			} catch (error) {
				lastError = error as Error

				const shouldRetry = this.shouldRetryError(error, attempt, maxAttempts)

				if (!shouldRetry || attempt >= maxAttempts) {
					throw error
				}

				const delay = this.calculateDelay(attempt)

				console.warn(
					`[${this.name}] Retrying request to ${context.path} (attempt ${attempt + 1}/${maxAttempts}) after ${delay}ms`,
				)

				await this.sleep(delay)
			}
		}

		// This shouldn't be reached, but TypeScript requires it
		throw lastError ?? new Error('Max retry attempts reached')
	}

	shouldApply(context: RequestContext): boolean {
		if (context.options.retry === false) {
			return false
		}

		return super.shouldApply(context)
	}

	/**
	 * Determine if an error should be retried
	 */
	private shouldRetryError(error: unknown, attempt: number, _maxAttempts: number): boolean {
		if (this.config.shouldRetry) {
			return this.config.shouldRetry(error, attempt)
		}

		if (this.config.retryOnNetworkError && this.isNetworkError(error)) {
			return true
		}

		if (error instanceof ModrinthApiError && error.statusCode) {
			return this.config.retryableStatusCodes.includes(error.statusCode)
		}

		return false
	}

	/**
	 * Check if an error is a network error
	 */
	private isNetworkError(error: unknown): boolean {
		// Common network error indicators
		const networkErrorPatterns = [
			/network/i,
			/timeout/i,
			/ECONNREFUSED/i,
			/ENOTFOUND/i,
			/ETIMEDOUT/i,
			/ECONNRESET/i,
		]

		const errorMessage = error instanceof Error ? error.message : String(error)
		return networkErrorPatterns.some((pattern) => pattern.test(errorMessage))
	}

	/**
	 * Get max attempts for this request
	 */
	private getMaxAttempts(context: RequestContext): number {
		if (typeof context.options.retry === 'number') {
			return context.options.retry
		}

		return this.config.maxAttempts
	}

	/**
	 * Calculate delay before next retry based on backoff strategy
	 */
	private calculateDelay(attempt: number): number {
		const { backoffStrategy, initialDelay, maxDelay } = this.config

		let delay: number

		switch (backoffStrategy) {
			case 'exponential':
				// Exponential: delay = initialDelay * 2^(attempt-1)
				delay = initialDelay * Math.pow(2, attempt - 1)
				break

			case 'linear':
				// Linear: delay = initialDelay * attempt
				delay = initialDelay * attempt
				break

			case 'constant':
				// Constant: delay = initialDelay
				delay = initialDelay
				break

			default:
				delay = initialDelay
		}

		// Add jitter (random 0-1000ms) to prevent thundering herd
		delay += Math.random() * 1000

		return Math.min(delay, maxDelay)
	}

	/**
	 * Sleep for a given duration
	 */
	private sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms))
	}
}
