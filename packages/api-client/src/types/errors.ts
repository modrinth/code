/**
 * Data for API errors
 */
export type ApiErrorData = {
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
export type ModrinthErrorResponse = {
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
	if (typeof obj !== 'object' || obj === null) {
		return false
	}
	const record = obj as Record<string, unknown>
	return typeof record.error === 'string' && typeof record.description === 'string'
}
