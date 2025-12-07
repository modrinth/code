/**
 * HTTP method types
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'

/**
 * Options for making a request
 */
export type RequestOptions = {
	/**
	 * API to use for this request
	 * - 'labrinth': Main Modrinth API (resolves to labrinthBaseUrl)
	 * - 'archon': Modrinth Servers API (resolves to archonBaseUrl)
	 * - string: Custom base URL (e.g., 'https://custom-api.com')
	 */
	api: 'labrinth' | 'archon' | string

	/**
	 * API version to use
	 * - number: version number (e.g., 2 for v2, 3 for v3)
	 * - 'internal': use internal API
	 */
	version: number | 'internal' | string

	/**
	 * HTTP method to use
	 * @default 'GET'
	 */
	method?: HttpMethod

	/**
	 * Request headers
	 */
	headers?: Record<string, string>

	/**
	 * Request body (will be JSON stringified if object)
	 */
	body?: unknown

	/**
	 * URL query parameters
	 */
	params?: Record<string, unknown>

	/**
	 * Request timeout in milliseconds
	 */
	timeout?: number

	/**
	 * Abort signal for cancelling requests
	 */
	signal?: AbortSignal

	/**
	 * Retry configuration for this specific request
	 * - false: no retries
	 * - true: use default retry config
	 * - number: max retry attempts
	 */
	retry?: boolean | number

	/**
	 * Circuit breaker configuration for this specific request
	 * - false: disable circuit breaker
	 * - true: use default circuit breaker config
	 */
	circuitBreaker?: boolean

	/**
	 * Whether to skip authentication for this request
	 * @default false
	 */
	skipAuth?: boolean
}

/**
 * Full context passed to features during request execution
 */
export type RequestContext = {
	/**
	 * Full URL being requested (with base URL and versioning applied)
	 */
	url: string

	/**
	 * Original path (before base URL and versioning)
	 */
	path: string

	/**
	 * Request options
	 */
	options: RequestOptions

	/**
	 * Current attempt number (1-indexed)
	 */
	attempt: number

	/**
	 * Timestamp when request started
	 */
	startTime: number

	/**
	 * Additional metadata that features can attach
	 */
	metadata?: Record<string, unknown>
}

/**
 * Generic response wrapper
 */
export type ResponseData<T = unknown> = {
	/**
	 * Response data
	 */
	data: T

	/**
	 * HTTP status code
	 */
	status: number

	/**
	 * Response headers
	 */
	headers: Record<string, string>
}
