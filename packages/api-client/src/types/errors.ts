/**
 * Data for API errors
 */
export interface ApiErrorData {
	/**
	 * HTTP status code (if available)
	 */
	statusCode?: number

	/**
	 * Original error that was caught
	 */
	originalError?: Error

	/**
	 * Response data from the API (if available)
	 */
	responseData?: unknown

	/**
	 * Error context (e.g., module name, operation being performed)
	 */
	context?: string
}

/**
 * Modrinth V1 error response format
 * Used by kyros + archon APIs
 */
export interface ModrinthErrorResponse {
	/**
	 * Error code/identifier
	 */
	error: string

	/**
	 * Human-readable error description
	 */
	description: string

	/**
	 * Optional context about where the error occurred
	 */
	context?: string
}

/**
 * Type guard to check if an object is a ModrinthErrorResponse
 */
export function isModrinthErrorResponse(obj: unknown): obj is ModrinthErrorResponse {
	return (
		typeof obj === 'object' &&
		obj !== null &&
		typeof obj.error === 'string' &&
		typeof obj.description === 'string'
	)
}
